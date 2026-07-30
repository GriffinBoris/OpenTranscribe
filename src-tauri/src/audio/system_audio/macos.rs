use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Sender, bounded};
use opentranscribe_domain::CaptureDevice;
use screencapturekit::prelude::*;

use crate::audio::packet_writer::{
    AudioFormat, AudioPacket, CaptureSignals, PACKET_QUEUE_CAPACITY, enqueue_samples, write_chunks,
};
use crate::error::{AppError, AppResult};

const SYSTEM_SAMPLE_RATE: u32 = 48_000;
const SYSTEM_CHANNELS: u16 = 2;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub fn availability() -> bool {
    true
}

pub fn permission_granted() -> Option<bool> {
    // SAFETY: CoreGraphics exposes this parameterless process-level permission check.
    Some(unsafe { CGPreflightScreenCaptureAccess() })
}

pub struct SystemAudioCapture {
    pub device: CaptureDevice,
    stop_sender: Sender<()>,
    capture_thread: JoinHandle<AppResult<()>>,
    writer_thread: JoinHandle<AppResult<()>>,
}

impl SystemAudioCapture {
    pub fn start(recovery_directory: PathBuf, signals: CaptureSignals) -> AppResult<Self> {
        let (packet_sender, packet_receiver) = bounded(PACKET_QUEUE_CAPACITY);
        let (stop_sender, stop_receiver) = bounded(1);
        let (ready_sender, ready_receiver) = bounded(1);
        let writer_directory = recovery_directory.clone();
        let writer_thread = thread::spawn(move || {
            write_chunks(
                packet_receiver,
                &writer_directory,
                "system",
                AudioFormat {
                    channels: SYSTEM_CHANNELS,
                    sample_rate: SYSTEM_SAMPLE_RATE,
                },
            )
        });
        let capture_thread =
            thread::spawn(move || run_capture(packet_sender, stop_receiver, ready_sender, signals));

        match ready_receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(())) => Ok(Self {
                device: CaptureDevice {
                    stable_id: "macos-system-output".to_owned(),
                    label: "System Default Output".to_owned(),
                    sample_rate_hz: SYSTEM_SAMPLE_RATE,
                    channels: SYSTEM_CHANNELS,
                },
                stop_sender,
                capture_thread,
                writer_thread,
            }),
            Ok(Err(message)) => {
                let _ = stop_sender.send(());
                let capture_result = join_capture(capture_thread, "system capture");
                let writer_result = join_capture(writer_thread, "system audio writer");
                writer_result?;

                if let Err(error) = capture_result {
                    log::debug!("system audio startup stopped with the reported error: {error}");
                }

                Err(AppError::Audio(message))
            }
            Err(error) => {
                let _ = stop_sender.send(());
                let capture_result = join_capture(capture_thread, "system capture");
                let writer_result = join_capture(writer_thread, "system audio writer");
                capture_result?;
                writer_result?;
                Err(AppError::Audio(format!(
                    "system audio did not start in time: {error}"
                )))
            }
        }
    }

    pub fn stop(self) -> AppResult<()> {
        let _ = self.stop_sender.send(());
        let capture_result = join_capture(self.capture_thread, "system capture");
        let writer_result = join_capture(self.writer_thread, "system audio writer");
        capture_result?;
        writer_result
    }
}

fn run_capture(
    packet_sender: Sender<AudioPacket>,
    stop_receiver: crossbeam_channel::Receiver<()>,
    ready_sender: Sender<Result<(), String>>,
    signals: CaptureSignals,
) -> AppResult<()> {
    let content = match SCShareableContent::get() {
        Ok(content) => content,
        Err(error) => {
            let message = content_access_message(&error);
            let _ = ready_sender.send(Err(message.clone()));
            return Err(AppError::Audio(message));
        }
    };
    let display = match content.displays().into_iter().next() {
        Some(display) => display,
        None => {
            let message = "no display is available for system audio capture".to_owned();
            let _ = ready_sender.send(Err(message.clone()));
            return Err(AppError::Audio(message));
        }
    };
    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();
    let configuration = SCStreamConfiguration::new()
        .with_width(2)
        .with_height(2)
        .with_shows_cursor(false)
        .with_captures_audio(true)
        .with_sample_rate(SYSTEM_SAMPLE_RATE as i32)
        .with_channel_count(SYSTEM_CHANNELS as i32)
        .with_excludes_current_process_audio(true);
    let handler = SystemAudioHandler {
        packet_sender,
        signals,
    };
    let mut stream = SCStream::new(&filter, &configuration);
    stream.add_output_handler(handler, SCStreamOutputType::Audio);

    if let Err(error) = stream.start_capture() {
        let message = stream_start_message(&error);
        let _ = ready_sender.send(Err(message.clone()));
        return Err(AppError::Audio(message));
    }

    if ready_sender.send(Ok(())).is_err() {
        stream.stop_capture().map_err(system_error)?;
        return Ok(());
    }

    let _ = stop_receiver.recv();
    stream.stop_capture().map_err(system_error)
}

struct SystemAudioHandler {
    packet_sender: Sender<AudioPacket>,
    signals: CaptureSignals,
}

impl SCStreamOutputTrait for SystemAudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if !matches!(output_type, SCStreamOutputType::Audio) {
            return;
        }

        let Some(samples) = float_samples(&sample) else {
            self.signals.dropped_packets.fetch_add(1, Ordering::Relaxed);
            return;
        };

        enqueue_samples(samples, &self.packet_sender, &self.signals);
    }
}

fn float_samples(sample: &CMSampleBuffer) -> Option<Vec<f32>> {
    let description = sample.format_description()?;

    if !description.is_pcm()
        || !description.audio_is_float()
        || description.audio_is_big_endian()
        || description.audio_bits_per_channel() != Some(32)
        || description.audio_channel_count() != Some(u32::from(SYSTEM_CHANNELS))
        || description.audio_sample_rate()? as u32 != SYSTEM_SAMPLE_RATE
    {
        return None;
    }

    let buffers = sample.audio_buffer_list()?;

    if buffers.num_buffers() == 1 {
        return decode_float_bytes(buffers.buffer(0)?.data());
    }

    if buffers.num_buffers() != usize::from(SYSTEM_CHANNELS) {
        return None;
    }

    let channels = (0..buffers.num_buffers())
        .map(|index| {
            let buffer = buffers.get(index)?;

            if buffer.number_channels != 1 {
                return None;
            }

            decode_float_bytes(buffer.data())
        })
        .collect::<Option<Vec<_>>>()?;
    let frame_count = channels.iter().map(Vec::len).min()?;
    let mut samples = Vec::with_capacity(frame_count * channels.len());

    for frame in 0..frame_count {
        for channel in &channels {
            samples.push(channel[frame]);
        }
    }

    Some(samples)
}

fn decode_float_bytes(bytes: &[u8]) -> Option<Vec<f32>> {
    let chunks = bytes.chunks_exact(size_of::<f32>());

    if !chunks.remainder().is_empty() {
        return None;
    }

    Some(
        chunks
            .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("chunk size is exact")))
            .collect(),
    )
}

fn content_access_message(error: impl std::fmt::Display) -> String {
    format!(
        "ScreenCaptureKit could not access system audio: {error}. Allow OpenTranscribe in System Settings → Privacy & Security → Screen & System Audio Recording, then relaunch it"
    )
}

fn stream_start_message(error: impl std::fmt::Display) -> String {
    format!("ScreenCaptureKit could not start system audio capture: {error}")
}

fn system_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(format!("system audio capture failed: {error}"))
}

fn join_capture(handle: JoinHandle<AppResult<()>>, description: &str) -> AppResult<()> {
    handle
        .join()
        .map_err(|_| AppError::Audio(format!("{description} thread crashed")))?
}

#[cfg(test)]
mod tests {
    use super::{content_access_message, decode_float_bytes, stream_start_message};

    #[test]
    fn decodes_little_endian_float_samples() {
        let bytes = [0.25_f32.to_le_bytes(), (-0.5_f32).to_le_bytes()].concat();
        assert_eq!(decode_float_bytes(&bytes), Some(vec![0.25, -0.5]));
    }

    #[test]
    fn rejects_incomplete_float_samples() {
        assert_eq!(decode_float_bytes(&[0, 1, 2]), None);
    }

    #[test]
    fn preserves_content_access_error_context() {
        let message = content_access_message("permission denied");

        assert!(message.contains("permission denied"));
        assert!(message.contains("Screen & System Audio Recording"));
        assert!(message.contains("relaunch"));
    }

    #[test]
    fn preserves_stream_start_error_context_without_calling_it_a_permission_error() {
        let message = stream_start_message("invalid stream configuration");

        assert_eq!(
            message,
            "ScreenCaptureKit could not start system audio capture: invalid stream configuration"
        );
    }
}
