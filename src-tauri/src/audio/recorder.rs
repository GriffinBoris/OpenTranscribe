use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait};
use opentranscribe_domain::CaptureDevice;
use serde::{Deserialize, Serialize};

use super::microphone::MicrophoneCapture;
use super::packet_writer::CaptureSignals;
use super::system_audio::{
    SystemAudioCapture, availability as system_audio_availability,
    permission_granted as system_audio_permission_granted,
};
use crate::error::{AppError, AppResult};
use crate::transcription::LiveAudioSink;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioDevices {
    pub microphones: Vec<AudioDevice>,
    pub system_audio_available: bool,
    pub system_audio_permission_granted: Option<bool>,
    pub system_audio_permission_settings_available: bool,
}

#[derive(Clone)]
pub struct StartRecordingOptions {
    pub session_id: String,
    pub microphone_device_id: Option<String>,
    pub capture_system_audio: bool,
    pub microphone_live_audio: Option<LiveAudioSink>,
    pub system_live_audio: Option<LiveAudioSink>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecordingStatus {
    pub session_id: String,
    pub is_paused: bool,
    pub captures_system_audio: bool,
    pub elapsed_ms: u64,
    pub microphone_peak: f32,
    pub system_peak: f32,
    pub dropped_packets: u64,
}

#[derive(Clone, Debug)]
pub struct RecordingCapture {
    pub microphone: CaptureDevice,
    pub system_output: Option<CaptureDevice>,
}

struct ActiveRecording {
    session_id: String,
    started: Instant,
    paused_at: Option<Instant>,
    paused_duration: Duration,
    paused: Arc<AtomicBool>,
    microphone_peak: Arc<AtomicU32>,
    system_peak: Arc<AtomicU32>,
    dropped_packets: Arc<AtomicU64>,
    microphone: MicrophoneCapture,
    system_audio: Option<SystemAudioCapture>,
}

#[derive(Default)]
pub struct RecordingController {
    active: Mutex<Option<ActiveRecording>>,
}

impl RecordingController {
    pub fn devices(&self) -> AppResult<AudioDevices> {
        let host = cpal::default_host();
        let default_id = host
            .default_input_device()
            .and_then(|device| device.id().ok())
            .map(|id| id.to_string());
        let microphones = host
            .input_devices()
            .map_err(audio_error)?
            .enumerate()
            .map(|(index, device)| {
                let label = device
                    .description()
                    .map(|description| description.name().to_owned())
                    .unwrap_or_else(|_| format!("Microphone {}", index + 1));
                let id = device
                    .id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|_| label.clone());
                AudioDevice {
                    is_default: default_id.as_ref() == Some(&id),
                    id,
                    label,
                }
            })
            .collect();

        Ok(AudioDevices {
            microphones,
            system_audio_available: system_audio_availability(),
            system_audio_permission_granted: system_audio_permission_granted(),
            system_audio_permission_settings_available: cfg!(target_os = "macos"),
        })
    }

    pub fn start(
        &self,
        options: StartRecordingOptions,
        recovery_directory: PathBuf,
    ) -> AppResult<RecordingCapture> {
        let mut active = self.active.lock().expect("recording lock poisoned");

        if active.is_some() {
            return Err(AppError::RecordingActive);
        }

        fs::create_dir_all(&recovery_directory)?;
        let host = cpal::default_host();
        let device = select_input_device(&host, options.microphone_device_id.as_deref())?;
        let supported_config = device.default_input_config().map_err(audio_error)?;
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();
        let paused = Arc::new(AtomicBool::new(true));
        let microphone_peak = Arc::new(AtomicU32::new(0));
        let system_peak = Arc::new(AtomicU32::new(0));
        let dropped_packets = Arc::new(AtomicU64::new(0));
        let system_audio = options
            .capture_system_audio
            .then(|| {
                SystemAudioCapture::start(
                    recovery_directory.clone(),
                    CaptureSignals {
                        paused: Arc::clone(&paused),
                        peak: Arc::clone(&system_peak),
                        dropped_packets: Arc::clone(&dropped_packets),
                        live_audio: options.system_live_audio.clone(),
                    },
                )
            })
            .transpose()?;
        let system_output = system_audio.as_ref().map(|capture| capture.device.clone());
        let microphone_device = CaptureDevice {
            stable_id: device.id().map_err(audio_error)?.to_string(),
            label: device.description().map_err(audio_error)?.name().to_owned(),
            sample_rate_hz: config.sample_rate,
            channels: config.channels,
        };
        let microphone = match MicrophoneCapture::start(
            device,
            config,
            sample_format,
            recovery_directory,
            CaptureSignals {
                paused: Arc::clone(&paused),
                peak: Arc::clone(&microphone_peak),
                dropped_packets: Arc::clone(&dropped_packets),
                live_audio: options.microphone_live_audio,
            },
        ) {
            Ok(capture) => capture,
            Err(error) => {
                if let Some(system_audio) = system_audio
                    && let Err(stop_error) = system_audio.stop()
                {
                    log::warn!(
                        "system audio cleanup failed after microphone start failure: {stop_error}"
                    );
                }
                return Err(error);
            }
        };
        let capture = RecordingCapture {
            microphone: microphone_device,
            system_output,
        };
        paused.store(false, Ordering::Relaxed);

        *active = Some(ActiveRecording {
            session_id: options.session_id,
            started: Instant::now(),
            paused_at: None,
            paused_duration: Duration::ZERO,
            paused,
            microphone_peak,
            system_peak,
            dropped_packets,
            microphone,
            system_audio,
        });
        Ok(capture)
    }

    pub fn pause(&self, paused: bool) -> AppResult<RecordingStatus> {
        let mut active = self.active.lock().expect("recording lock poisoned");
        let active = active.as_mut().ok_or(AppError::RecordingInactive)?;

        if paused && active.paused_at.is_none() {
            active.paused_at = Some(Instant::now());
        }

        if !paused && let Some(paused_at) = active.paused_at.take() {
            active.paused_duration += paused_at.elapsed();
        }

        active.paused.store(paused, Ordering::Relaxed);
        Ok(status_for(active))
    }

    pub fn status(&self) -> Option<RecordingStatus> {
        self.active
            .lock()
            .expect("recording lock poisoned")
            .as_ref()
            .map(status_for)
    }

    pub fn stop(&self) -> AppResult<RecordingStatus> {
        let active = self
            .active
            .lock()
            .expect("recording lock poisoned")
            .take()
            .ok_or(AppError::RecordingInactive)?;
        let status = status_for(&active);
        let microphone_result = active.microphone.stop();
        let system_result = active
            .system_audio
            .map(SystemAudioCapture::stop)
            .transpose();

        if let Err(error) = microphone_result {
            log::warn!("microphone stopped with an error; recovery chunks were preserved: {error}");
        }

        if let Err(error) = system_result {
            log::warn!(
                "system audio stopped with an error; recovery chunks were preserved: {error}"
            );
        }

        Ok(status)
    }
}

fn select_input_device(host: &cpal::Host, requested: Option<&str>) -> AppResult<cpal::Device> {
    if let Some(requested) = requested {
        return host
            .input_devices()
            .map_err(audio_error)?
            .find(|device| {
                device
                    .id()
                    .map(|id| id.to_string() == requested)
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                AppError::Audio(format!("microphone is no longer available: {requested}"))
            });
    }

    host.default_input_device()
        .ok_or_else(|| AppError::Audio("no default microphone is available".to_owned()))
}

fn status_for(active: &ActiveRecording) -> RecordingStatus {
    let elapsed = recorded_elapsed(
        active.started,
        active.paused_duration,
        active.paused_at,
        Instant::now(),
    );

    RecordingStatus {
        session_id: active.session_id.clone(),
        is_paused: active.paused.load(Ordering::Relaxed),
        captures_system_audio: active.system_audio.is_some(),
        elapsed_ms: elapsed.as_millis() as u64,
        microphone_peak: f32::from_bits(active.microphone_peak.load(Ordering::Relaxed)),
        system_peak: f32::from_bits(active.system_peak.load(Ordering::Relaxed)),
        dropped_packets: active.dropped_packets.load(Ordering::Relaxed),
    }
}

fn recorded_elapsed(
    started: Instant,
    paused_duration: Duration,
    paused_at: Option<Instant>,
    now: Instant,
) -> Duration {
    let current_pause = paused_at.map(|started| now - started).unwrap_or_default();
    (now - started).saturating_sub(paused_duration + current_pause)
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::recorded_elapsed;

    #[test]
    fn excludes_completed_and_active_pause_time_from_recording_elapsed_time() {
        let now = Instant::now();
        let elapsed = recorded_elapsed(
            now - Duration::from_secs(100),
            Duration::from_secs(20),
            Some(now - Duration::from_secs(30)),
            now,
        );

        assert_eq!(elapsed, Duration::from_secs(50));
    }
}
