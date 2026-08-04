use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, I24, SampleFormat, SizedSample, StreamConfig, U24};
use crossbeam_channel::{Sender, bounded};

use super::packet_writer::{
    AudioFormat, AudioPacket, CaptureSignals, PACKET_QUEUE_CAPACITY, enqueue_samples, write_chunks,
};
use crate::error::{AppError, AppResult};

pub struct CpalCapture {
    stop_sender: Sender<()>,
    capture_thread: JoinHandle<AppResult<()>>,
    writer_thread: JoinHandle<AppResult<()>>,
}

#[derive(Clone, Copy)]
pub enum ChannelLayout {
    DualMono,
    #[cfg(target_os = "windows")]
    Preserve,
}

#[derive(Clone, Copy)]
struct CaptureFormat {
    source: AudioFormat,
    output: AudioFormat,
    layout: ChannelLayout,
}

impl CpalCapture {
    pub fn start(
        device: cpal::Device,
        config: StreamConfig,
        sample_format: SampleFormat,
        recovery_directory: PathBuf,
        track_name: &'static str,
        signals: CaptureSignals,
        channel_layout: ChannelLayout,
    ) -> AppResult<Self> {
        let (packet_sender, packet_receiver) = bounded(PACKET_QUEUE_CAPACITY);
        let (stop_sender, stop_receiver) = bounded(1);
        let (ready_sender, ready_receiver) = bounded(1);
        let source_format = AudioFormat {
            channels: config.channels,
            sample_rate: config.sample_rate,
        };
        let writer_config = match channel_layout {
            ChannelLayout::DualMono => AudioFormat {
                channels: 2,
                sample_rate: config.sample_rate,
            },
            #[cfg(target_os = "windows")]
            ChannelLayout::Preserve => source_format,
        };
        let capture_format = CaptureFormat {
            source: source_format,
            output: writer_config,
            layout: channel_layout,
        };
        let writer_thread = thread::spawn(move || {
            write_chunks(
                packet_receiver,
                &recovery_directory,
                track_name,
                writer_config,
            )
        });
        let capture_thread = thread::spawn(move || {
            let flush_sender = packet_sender.clone();
            let flush_signals = signals.clone();
            let stream = build_stream(
                &device,
                config,
                sample_format,
                packet_sender,
                signals,
                capture_format,
            );

            match stream {
                Ok(stream) => {
                    stream.play().map_err(audio_error)?;
                    let _ = ready_sender.send(Ok(()));
                    let _ = stop_receiver.recv();
                    drop(stream);
                    enqueue_capture_samples(
                        flush_signals.flush_processed_samples(),
                        &flush_sender,
                        &flush_signals,
                        capture_format,
                    );
                    Ok(())
                }
                Err(error) => {
                    let message = error.to_string();
                    let _ = ready_sender.send(Err(message));
                    Err(error)
                }
            }
        });

        match ready_receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self {
                stop_sender,
                capture_thread,
                writer_thread,
            }),
            Ok(Err(message)) => {
                let _ = stop_sender.send(());
                let capture_result = join_capture(capture_thread, &format!("{track_name} capture"));
                let writer_result =
                    join_capture(writer_thread, &format!("{track_name} audio writer"));
                writer_result?;

                if let Err(error) = capture_result {
                    log::debug!("{track_name} startup stopped with the reported error: {error}");
                }

                Err(AppError::Audio(message))
            }
            Err(error) => {
                let _ = stop_sender.send(());
                let capture_result = join_capture(capture_thread, &format!("{track_name} capture"));
                let writer_result =
                    join_capture(writer_thread, &format!("{track_name} audio writer"));
                capture_result?;
                writer_result?;
                Err(AppError::Audio(format!(
                    "{track_name} did not start in time: {error}"
                )))
            }
        }
    }

    pub fn stop(self) -> AppResult<()> {
        let _ = self.stop_sender.send(());
        let capture_result = join_capture(self.capture_thread, "audio capture");
        let writer_result = join_capture(self.writer_thread, "audio writer");
        capture_result?;
        writer_result
    }
}

fn build_stream(
    device: &cpal::Device,
    config: StreamConfig,
    sample_format: SampleFormat,
    sender: Sender<AudioPacket>,
    signals: CaptureSignals,
    capture_format: CaptureFormat,
) -> AppResult<cpal::Stream> {
    let stream = match sample_format {
        SampleFormat::I8 => {
            build_typed_stream::<i8>(device, config, sender, signals, capture_format)
        }
        SampleFormat::I16 => {
            build_typed_stream::<i16>(device, config, sender, signals, capture_format)
        }
        SampleFormat::I24 => {
            build_typed_stream::<I24>(device, config, sender, signals, capture_format)
        }
        SampleFormat::I32 => {
            build_typed_stream::<i32>(device, config, sender, signals, capture_format)
        }
        SampleFormat::I64 => {
            build_typed_stream::<i64>(device, config, sender, signals, capture_format)
        }
        SampleFormat::U8 => {
            build_typed_stream::<u8>(device, config, sender, signals, capture_format)
        }
        SampleFormat::U16 => {
            build_typed_stream::<u16>(device, config, sender, signals, capture_format)
        }
        SampleFormat::U24 => {
            build_typed_stream::<U24>(device, config, sender, signals, capture_format)
        }
        SampleFormat::U32 => {
            build_typed_stream::<u32>(device, config, sender, signals, capture_format)
        }
        SampleFormat::U64 => {
            build_typed_stream::<u64>(device, config, sender, signals, capture_format)
        }
        SampleFormat::F32 => {
            build_typed_stream::<f32>(device, config, sender, signals, capture_format)
        }
        SampleFormat::F64 => {
            build_typed_stream::<f64>(device, config, sender, signals, capture_format)
        }
        _ => {
            return Err(AppError::Audio(format!(
                "unsupported audio format: {sample_format:?}"
            )));
        }
    }
    .map_err(audio_error)?;
    Ok(stream)
}

fn build_typed_stream<T>(
    device: &cpal::Device,
    config: StreamConfig,
    sender: Sender<AudioPacket>,
    signals: CaptureSignals,
    capture_format: CaptureFormat,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream(
        config,
        move |samples: &[T], _| {
            let samples = convert_samples(samples);
            let samples = signals.process_samples(samples, capture_format.source);
            enqueue_capture_samples(samples, &sender, &signals, capture_format);
        },
        |error| log::error!("audio input stream error: {error}"),
        None,
    )
}

fn enqueue_capture_samples(
    samples: Vec<f32>,
    sender: &Sender<AudioPacket>,
    signals: &CaptureSignals,
    capture_format: CaptureFormat,
) {
    if samples.is_empty() {
        return;
    }

    let samples = match capture_format.layout {
        ChannelLayout::DualMono => dual_mono_samples(&samples, capture_format.source.channels),
        #[cfg(target_os = "windows")]
        ChannelLayout::Preserve => samples,
    };
    enqueue_samples(samples, sender, signals, capture_format.output);
}

fn dual_mono_samples(samples: &[f32], source_channels: u16) -> Vec<f32> {
    samples
        .chunks_exact(usize::from(source_channels))
        .flat_map(|frame| {
            let mono = frame.iter().sum::<f32>() / frame.len() as f32;
            [mono, mono]
        })
        .collect()
}

fn convert_samples<T>(samples: &[T]) -> Vec<f32>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    samples
        .iter()
        .map(|sample| f32::from_sample_(*sample))
        .collect()
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

fn join_capture(handle: JoinHandle<AppResult<()>>, description: &str) -> AppResult<()> {
    handle
        .join()
        .map_err(|_| AppError::Audio(format!("{description} thread crashed")))?
}

#[cfg(test)]
mod tests {
    use super::{convert_samples, dual_mono_samples};

    #[test]
    fn converts_signed_unsigned_and_float_device_samples() {
        assert_eq!(convert_samples(&[i32::MIN, 0, i32::MAX])[1], 0.0);
        assert_eq!(convert_samples(&[u32::MIN, 1_u32 << 31, u32::MAX])[1], 0.0);

        let floats = convert_samples(&[-1.0_f64, 0.25, 1.0]);
        assert_eq!(floats, vec![-1.0, 0.25, 1.0]);
    }

    #[test]
    fn normalizes_microphone_audio_to_dual_mono() {
        assert_eq!(
            dual_mono_samples(&[0.25, 0.75, -0.5, 0.5], 2),
            vec![0.5, 0.5, 0.0, 0.0]
        );
    }
}
