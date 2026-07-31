use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
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

impl CpalCapture {
    pub fn start(
        device: cpal::Device,
        config: StreamConfig,
        sample_format: SampleFormat,
        recovery_directory: PathBuf,
        track_name: &'static str,
        signals: CaptureSignals,
    ) -> AppResult<Self> {
        let (packet_sender, packet_receiver) = bounded(PACKET_QUEUE_CAPACITY);
        let (stop_sender, stop_receiver) = bounded(1);
        let (ready_sender, ready_receiver) = bounded(1);
        let writer_config = AudioFormat {
            channels: config.channels,
            sample_rate: config.sample_rate,
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
            let stream = build_stream(&device, &config, sample_format, packet_sender, signals);

            match stream {
                Ok(stream) => {
                    stream.play().map_err(audio_error)?;
                    let _ = ready_sender.send(Ok(()));
                    let _ = stop_receiver.recv();
                    drop(stream);
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
    config: &StreamConfig,
    sample_format: SampleFormat,
    sender: Sender<AudioPacket>,
    signals: CaptureSignals,
) -> AppResult<cpal::Stream> {
    let error_callback = |error| log::error!("audio input stream error: {error}");
    let capture_format = AudioFormat {
        channels: config.channels,
        sample_rate: config.sample_rate,
    };
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |samples: &[f32], _| {
                enqueue_samples(samples.to_vec(), &sender, &signals, capture_format);
            },
            error_callback,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |samples: &[i16], _| {
                let samples = samples
                    .iter()
                    .map(|sample| f32::from(*sample) / f32::from(i16::MAX))
                    .collect();
                enqueue_samples(samples, &sender, &signals, capture_format);
            },
            error_callback,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            config,
            move |samples: &[u16], _| {
                let samples = samples
                    .iter()
                    .map(|sample| (f32::from(*sample) / f32::from(u16::MAX)) * 2.0 - 1.0)
                    .collect();
                enqueue_samples(samples, &sender, &signals, capture_format);
            },
            error_callback,
            None,
        ),
        _ => {
            return Err(AppError::Audio(format!(
                "unsupported audio format: {sample_format:?}"
            )));
        }
    }
    .map_err(audio_error)?;
    Ok(stream)
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

fn join_capture(handle: JoinHandle<AppResult<()>>, description: &str) -> AppResult<()> {
    handle
        .join()
        .map_err(|_| AppError::Audio(format!("{description} thread crashed")))?
}
