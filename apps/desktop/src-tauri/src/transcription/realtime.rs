use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use http::header::{AUTHORIZATION, HeaderValue};
use opentranscribe_domain::{AppEvent, AudioSource, LiveTranscriptUpdate};
use serde_json::{Value, json};
use tauri::Manager;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::events::send_event;
use crate::state::AppState;

const REALTIME_URL: &str = "wss://api.openai.com/v1/realtime?intent=transcription";
const REALTIME_SAMPLE_RATE: u32 = 24_000;
const AUDIO_QUEUE_CAPACITY: usize = 64;
const MINIMUM_COMMIT_AUDIO: Duration = Duration::from_millis(100);
const SILENCE_COMMIT_INTERVAL: Duration = Duration::from_millis(600);
const MAXIMUM_COMMIT_AUDIO: Duration = Duration::from_secs(8);
const SILENCE_RMS_THRESHOLD: f32 = 0.012;

pub struct LiveAudioPacket {
    samples: Vec<f32>,
    channels: u16,
    sample_rate: u32,
}

#[derive(Clone)]
pub struct LiveAudioSink {
    sender: mpsc::Sender<LiveAudioPacket>,
}

impl LiveAudioSink {
    pub fn send(&self, samples: &[f32], channels: u16, sample_rate: u32) {
        let _ = self.sender.try_send(LiveAudioPacket {
            samples: samples.to_vec(),
            channels,
            sample_rate,
        });
    }
}

struct RealtimeWorker {
    stop_sender: oneshot::Sender<()>,
    thread: JoinHandle<()>,
}

pub struct OpenAiRealtimeController {
    microphone_sink: LiveAudioSink,
    system_sink: Option<LiveAudioSink>,
    workers: Vec<RealtimeWorker>,
}

impl OpenAiRealtimeController {
    pub fn start(
        app: tauri::AppHandle,
        session_id: String,
        api_key: String,
        language_hint: Option<String>,
        capture_system_audio: bool,
    ) -> Self {
        let (microphone_sink, microphone_worker) = spawn_worker(
            app.clone(),
            session_id.clone(),
            api_key.clone(),
            language_hint.clone(),
            AudioSource::Microphone,
        );
        let (system_sink, system_worker) = capture_system_audio
            .then(|| spawn_worker(app, session_id, api_key, language_hint, AudioSource::System))
            .unzip();
        let mut workers = vec![microphone_worker];

        if let Some(worker) = system_worker {
            workers.push(worker);
        }

        Self {
            microphone_sink,
            system_sink,
            workers,
        }
    }

    pub fn microphone_sink(&self) -> LiveAudioSink {
        self.microphone_sink.clone()
    }

    pub fn system_sink(&self) -> Option<LiveAudioSink> {
        self.system_sink.clone()
    }

    pub fn stop(self) {
        let mut threads = Vec::with_capacity(self.workers.len());

        for worker in self.workers {
            let _ = worker.stop_sender.send(());
            threads.push(worker.thread);
        }

        for thread in threads {
            if thread.join().is_err() {
                log::warn!("OpenAI live transcription worker crashed while stopping");
            }
        }
    }
}

fn spawn_worker(
    app: tauri::AppHandle,
    session_id: String,
    api_key: String,
    language_hint: Option<String>,
    source: AudioSource,
) -> (LiveAudioSink, RealtimeWorker) {
    let (audio_sender, audio_receiver) = mpsc::channel(AUDIO_QUEUE_CAPACITY);
    let (stop_sender, stop_receiver) = oneshot::channel();
    let thread = std::thread::Builder::new()
        .name(format!("openai-live-{}", source_name(&source)))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("OpenAI live transcription runtime must start");

            if let Err(error) = runtime.block_on(run_realtime_session(
                app.clone(),
                session_id,
                api_key,
                language_hint,
                source,
                audio_receiver,
                stop_receiver,
            )) {
                let message = format!("Live transcription stopped: {error}");
                log::warn!("{message}");
                send_event(
                    &app.state::<AppState>(),
                    AppEvent::AttentionRequired(message),
                );
            }
        })
        .expect("OpenAI live transcription worker must start");

    (
        LiveAudioSink {
            sender: audio_sender,
        },
        RealtimeWorker {
            stop_sender,
            thread,
        },
    )
}

async fn run_realtime_session(
    app: tauri::AppHandle,
    session_id: String,
    api_key: String,
    language_hint: Option<String>,
    source: AudioSource,
    mut audio_receiver: mpsc::Receiver<LiveAudioPacket>,
    mut stop_receiver: oneshot::Receiver<()>,
) -> Result<(), String> {
    let mut request = REALTIME_URL
        .into_client_request()
        .map_err(|error| error.to_string())?;
    request.headers_mut().insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|error| error.to_string())?,
    );
    let (socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|error| error.to_string())?;
    let (mut writer, mut reader) = socket.split();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match reader.next().await {
                Some(Ok(Message::Text(text))) => {
                    let event: Value =
                        serde_json::from_str(&text).map_err(|error| error.to_string())?;

                    match event["type"].as_str().unwrap_or_default() {
                        "session.created" => return Ok::<(), String>(()),
                        "error" => {
                            return Err(event["error"]["message"]
                                .as_str()
                                .unwrap_or("OpenAI rejected the transcription session")
                                .to_owned());
                        }
                        _ => {}
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(error.to_string()),
                None => return Err("OpenAI closed the transcription session".to_owned()),
            }
        }
    })
    .await
    .map_err(|_| "OpenAI did not initialize the transcription session".to_owned())??;
    let mut transcription = json!({
        "model": "gpt-live-transcribe",
        "delay": "low"
    });

    if let Some(language_hint) = language_hint {
        transcription["languages"] = json!([language_hint]);
    }

    writer
        .send(Message::Text(
            json!({
                "type": "session.update",
                "session": {
                    "type": "transcription",
                    "audio": {
                        "input": {
                            "format": {
                                "type": "audio/pcm",
                                "rate": REALTIME_SAMPLE_RATE
                            },
                            "transcription": transcription,
                            "turn_detection": null
                        }
                    }
                }
            })
            .to_string()
            .into(),
        ))
        .await
        .map_err(|error| error.to_string())?;

    let mut item_text = HashMap::<String, String>::new();
    let mut stop_deadline: Option<Instant> = None;
    let mut buffered_audio_duration = Duration::ZERO;
    let mut buffered_silence_duration = Duration::ZERO;

    loop {
        if stop_deadline.is_none() && stop_receiver.try_recv().is_ok() {
            if buffered_audio_duration >= MINIMUM_COMMIT_AUDIO {
                writer
                    .send(Message::Text(
                        json!({ "type": "input_audio_buffer.commit" })
                            .to_string()
                            .into(),
                    ))
                    .await
                    .map_err(|error| error.to_string())?;
            }
            stop_deadline = Some(Instant::now() + Duration::from_secs(3));
        }

        if stop_deadline.is_none() {
            while let Ok(packet) = audio_receiver.try_recv() {
                let packet_duration = audio_packet_duration(&packet);
                if audio_packet_is_silent(&packet) {
                    buffered_silence_duration += packet_duration;
                } else {
                    buffered_silence_duration = Duration::ZERO;
                }
                let audio = pcm16_base64(packet);

                if audio.is_empty() {
                    continue;
                }

                writer
                    .send(Message::Text(
                        json!({
                            "type": "input_audio_buffer.append",
                            "audio": audio,
                        })
                        .to_string()
                        .into(),
                    ))
                    .await
                    .map_err(|error| error.to_string())?;
                buffered_audio_duration += packet_duration;
            }

            if should_commit_audio(buffered_audio_duration, buffered_silence_duration) {
                writer
                    .send(Message::Text(
                        json!({ "type": "input_audio_buffer.commit" })
                            .to_string()
                            .into(),
                    ))
                    .await
                    .map_err(|error| error.to_string())?;
                buffered_audio_duration = Duration::ZERO;
                buffered_silence_duration = Duration::ZERO;
            }
        }

        if stop_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            break;
        }

        match tokio::time::timeout(Duration::from_millis(20), reader.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                handle_server_event(&app, &session_id, &source, &text, &mut item_text)?;
            }
            Ok(Some(Ok(_))) | Err(_) => {}
            Ok(Some(Err(error))) => return Err(error.to_string()),
            Ok(None) => break,
        }
    }

    let _ = writer.close().await;
    Ok(())
}

fn handle_server_event(
    app: &tauri::AppHandle,
    session_id: &str,
    source: &AudioSource,
    message: &str,
    item_text: &mut HashMap<String, String>,
) -> Result<(), String> {
    let event: Value = serde_json::from_str(message).map_err(|error| error.to_string())?;
    let event_type = event["type"].as_str().unwrap_or_default();

    if event_type == "error" {
        return Err(event["error"]["message"]
            .as_str()
            .unwrap_or("OpenAI returned a realtime transcription error")
            .to_owned());
    }

    if !matches!(
        event_type,
        "conversation.item.input_audio_transcription.delta"
            | "conversation.item.input_audio_transcription.completed"
    ) {
        return Ok(());
    }

    let item_id = event["item_id"].as_str().unwrap_or_default();

    if item_id.is_empty() {
        return Ok(());
    }

    let completed = event_type.ends_with(".completed");
    let text = if completed {
        event["transcript"].as_str().unwrap_or_default().to_owned()
    } else {
        let text = item_text.entry(item_id.to_owned()).or_default();
        text.push_str(event["delta"].as_str().unwrap_or_default());
        text.clone()
    };

    if text.trim().is_empty() {
        return Ok(());
    }

    if completed {
        item_text.remove(item_id);
    }

    let started_at_ms = app
        .state::<AppState>()
        .recorder
        .status()
        .map(|status| status.elapsed_ms)
        .unwrap_or_default();
    send_event(
        &app.state::<AppState>(),
        AppEvent::LiveTranscriptChanged(LiveTranscriptUpdate {
            session_id: session_id.to_owned(),
            item_id: item_id.to_owned(),
            source: source.clone(),
            text,
            completed,
            started_at_ms,
        }),
    );
    Ok(())
}

fn pcm16_base64(packet: LiveAudioPacket) -> String {
    if packet.channels == 0 || packet.sample_rate == 0 {
        return String::new();
    }

    let mono = packet
        .samples
        .chunks(usize::from(packet.channels))
        .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
        .collect::<Vec<_>>();
    let output_len = mono.len() * REALTIME_SAMPLE_RATE as usize / packet.sample_rate as usize;
    let mut bytes = Vec::with_capacity(output_len * 2);

    for output_index in 0..output_len {
        let input_position =
            output_index as f64 * packet.sample_rate as f64 / REALTIME_SAMPLE_RATE as f64;
        let input_index = input_position.floor() as usize;
        let next_index = (input_index + 1).min(mono.len().saturating_sub(1));
        let fraction = (input_position - input_index as f64) as f32;
        let sample = mono.get(input_index).copied().unwrap_or_default()
            + (mono.get(next_index).copied().unwrap_or_default()
                - mono.get(input_index).copied().unwrap_or_default())
                * fraction;
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn audio_packet_duration(packet: &LiveAudioPacket) -> Duration {
    if packet.channels == 0 || packet.sample_rate == 0 {
        return Duration::ZERO;
    }

    let frame_count = packet.samples.len() / usize::from(packet.channels);
    Duration::from_secs_f64(frame_count as f64 / f64::from(packet.sample_rate))
}

fn audio_packet_is_silent(packet: &LiveAudioPacket) -> bool {
    if packet.samples.is_empty() {
        return true;
    }

    let mean_square = packet
        .samples
        .iter()
        .map(|sample| sample * sample)
        .sum::<f32>()
        / packet.samples.len() as f32;
    mean_square.sqrt() < SILENCE_RMS_THRESHOLD
}

fn should_commit_audio(buffered: Duration, silence: Duration) -> bool {
    buffered >= MAXIMUM_COMMIT_AUDIO
        || (buffered >= MINIMUM_COMMIT_AUDIO && silence >= SILENCE_COMMIT_INTERVAL)
}

fn source_name(source: &AudioSource) -> &'static str {
    match source {
        AudioSource::Microphone => "microphone",
        AudioSource::System => "system",
        AudioSource::Mixed => "mixed",
        AudioSource::Imported => "imported",
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        LiveAudioPacket, audio_packet_duration, audio_packet_is_silent, pcm16_base64,
        should_commit_audio,
    };

    #[test]
    fn converts_stereo_audio_to_24khz_pcm16() {
        let encoded = pcm16_base64(LiveAudioPacket {
            samples: vec![0.5, 0.5, -0.5, -0.5],
            channels: 2,
            sample_rate: 48_000,
        });
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .expect("PCM should decode");

        assert_eq!(bytes.len(), 2);
        assert_eq!(i16::from_le_bytes([bytes[0], bytes[1]]), 16_384);
    }

    #[test]
    fn does_not_commit_a_short_packet_after_resume() {
        assert!(!should_commit_audio(
            Duration::from_millis(21),
            Duration::from_millis(21),
        ));
    }

    #[test]
    fn commits_at_a_natural_pause_or_maximum_chunk_length() {
        assert!(should_commit_audio(
            Duration::from_secs(2),
            Duration::from_millis(600),
        ));
        assert!(should_commit_audio(Duration::from_secs(8), Duration::ZERO));
    }

    #[test]
    fn measures_packet_duration_and_silence() {
        let silence = LiveAudioPacket {
            samples: vec![0.0; 960],
            channels: 2,
            sample_rate: 48_000,
        };
        let speech = LiveAudioPacket {
            samples: vec![0.1; 960],
            channels: 2,
            sample_rate: 48_000,
        };

        assert_eq!(audio_packet_duration(&silence), Duration::from_millis(10));
        assert!(audio_packet_is_silent(&silence));
        assert!(!audio_packet_is_silent(&speech));
    }
}
