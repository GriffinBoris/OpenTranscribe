use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender, TrySendError};
use hound::{SampleFormat as WavSampleFormat, WavSpec, WavWriter};

use crate::error::{AppError, AppResult};
use crate::transcription::LiveAudioSink;

use super::echo_cancellation::AudioProcessor;

pub const CHUNK_SECONDS: u64 = 10;
pub const PACKET_QUEUE_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

pub struct AudioPacket {
    pub samples: Vec<f32>,
}

#[derive(Clone)]
pub struct CaptureSignals {
    pub paused: Arc<AtomicBool>,
    pub peak: Arc<AtomicU32>,
    pub dropped_packets: Arc<AtomicU64>,
    pub live_audio: Option<LiveAudioSink>,
    pub processor: Option<AudioProcessor>,
}

impl CaptureSignals {
    pub fn process_samples(&self, samples: Vec<f32>, format: AudioFormat) -> Vec<f32> {
        if let Some(processor) = &self.processor {
            processor.process(samples, format)
        } else {
            samples
        }
    }

    pub fn flush_processed_samples(&self) -> Vec<f32> {
        self.processor
            .as_ref()
            .map(AudioProcessor::flush)
            .unwrap_or_default()
    }
}

pub fn enqueue_samples(
    samples: Vec<f32>,
    sender: &Sender<AudioPacket>,
    signals: &CaptureSignals,
    format: AudioFormat,
) {
    if signals.paused.load(Ordering::Relaxed) {
        signals.peak.store(0, Ordering::Relaxed);
        return;
    }

    let packet_peak = samples
        .iter()
        .fold(0.0_f32, |current, sample| current.max(sample.abs()));
    signals.peak.store(packet_peak.to_bits(), Ordering::Relaxed);

    if let Some(live_audio) = &signals.live_audio {
        live_audio.send(&samples, format.channels, format.sample_rate);
    }

    if let Err(TrySendError::Full(_)) = sender.try_send(AudioPacket { samples }) {
        signals.dropped_packets.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn write_chunks(
    receiver: Receiver<AudioPacket>,
    recovery_directory: &Path,
    prefix: &str,
    format: AudioFormat,
) -> AppResult<()> {
    let samples_per_chunk =
        u64::from(format.sample_rate) * u64::from(format.channels) * CHUNK_SECONDS;
    let mut chunk_number = 1_u64;
    let mut samples_in_chunk = 0_u64;
    let mut writer = create_chunk(recovery_directory, prefix, chunk_number, format)?;

    for packet in receiver {
        for sample in packet.samples {
            if samples_in_chunk >= samples_per_chunk {
                writer.finalize().map_err(audio_error)?;
                chunk_number += 1;
                samples_in_chunk = 0;
                writer = create_chunk(recovery_directory, prefix, chunk_number, format)?;
            }

            let pcm_sample = (sample.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
            writer.write_sample(pcm_sample).map_err(audio_error)?;
            samples_in_chunk += 1;
        }
    }

    writer.finalize().map_err(audio_error)?;
    Ok(())
}

fn create_chunk(
    directory: &Path,
    prefix: &str,
    chunk_number: u64,
    format: AudioFormat,
) -> AppResult<WavWriter<std::io::BufWriter<fs::File>>> {
    let path = directory.join(format!("{prefix}-{chunk_number:06}.wav"));
    WavWriter::create(
        path,
        WavSpec {
            channels: format.channels,
            sample_rate: format.sample_rate,
            bits_per_sample: 24,
            sample_format: WavSampleFormat::Int,
        },
    )
    .map_err(audio_error)
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

#[cfg(test)]
mod tests {
    use crossbeam_channel::unbounded;
    use tempfile::tempdir;

    use super::{AudioFormat, AudioPacket, write_chunks};

    #[test]
    fn rotates_and_finalizes_recovery_chunks() {
        let directory = tempdir().expect("temporary directory should exist");
        let (sender, receiver) = unbounded();
        sender
            .send(AudioPacket {
                samples: vec![0.25; 150],
            })
            .expect("writer should be connected");
        drop(sender);

        write_chunks(
            receiver,
            directory.path(),
            "microphone",
            AudioFormat {
                channels: 1,
                sample_rate: 10,
            },
        )
        .expect("chunks should be written");

        let first = hound::WavReader::open(directory.path().join("microphone-000001.wav"))
            .expect("first chunk should be playable");
        let second = hound::WavReader::open(directory.path().join("microphone-000002.wav"))
            .expect("second chunk should be playable");
        assert_eq!(first.spec().bits_per_sample, 24);
        assert_eq!(first.len(), 100);
        assert_eq!(second.len(), 50);
    }
}
