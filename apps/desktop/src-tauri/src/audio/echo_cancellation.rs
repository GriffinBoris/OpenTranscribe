use std::sync::{Arc, Mutex};

use sonora::config::{EchoCanceller, MaxProcessingRate};
use sonora::{AudioProcessing, Config, StreamConfig};

use super::packet_writer::AudioFormat;

#[derive(Clone)]
pub(crate) enum AudioProcessor {
    Microphone(Arc<EchoCancellation>),
    System(Arc<EchoCancellation>),
}

impl AudioProcessor {
    pub(crate) fn microphone(cancellation: Arc<EchoCancellation>) -> Self {
        Self::Microphone(cancellation)
    }

    pub(crate) fn system(cancellation: Arc<EchoCancellation>) -> Self {
        Self::System(cancellation)
    }

    pub(crate) fn process(&self, samples: Vec<f32>, format: AudioFormat) -> Vec<f32> {
        let cancellation = match self {
            Self::Microphone(cancellation) | Self::System(cancellation) => cancellation,
        };
        let Ok(mut state) = cancellation.state.try_lock() else {
            return samples;
        };

        match self {
            Self::Microphone(_) => state.process_microphone(samples, format),
            Self::System(_) => state.process_system(samples, format),
        }
    }

    pub(crate) fn flush(&self) -> Vec<f32> {
        match self {
            Self::Microphone(cancellation) => cancellation
                .state
                .lock()
                .expect("echo cancellation lock poisoned")
                .flush_microphone(),
            Self::System(_) => Vec::new(),
        }
    }
}

pub(crate) struct EchoCancellation {
    state: Mutex<EchoCancellationState>,
}

impl EchoCancellation {
    pub(crate) fn new(microphone_format: AudioFormat, system_format: AudioFormat) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(EchoCancellationState::new(microphone_format, system_format)),
        })
    }
}

struct EchoCancellationState {
    processor: AudioProcessing,
    microphone_format: AudioFormat,
    system_format: AudioFormat,
    microphone_remainder: Vec<f32>,
    system_remainder: Vec<f32>,
}

impl EchoCancellationState {
    fn new(microphone_format: AudioFormat, system_format: AudioFormat) -> Self {
        let config = Config {
            pipeline: sonora::config::Pipeline {
                maximum_internal_processing_rate: MaxProcessingRate::Rate48kHz,
                ..Default::default()
            },
            echo_canceller: Some(EchoCanceller::default()),
            ..Default::default()
        };
        let processor = AudioProcessing::builder()
            .config(config)
            .capture_config(stream_config(microphone_format))
            .render_config(stream_config(system_format))
            .build();

        Self {
            processor,
            microphone_format,
            system_format,
            microphone_remainder: Vec::new(),
            system_remainder: Vec::new(),
        }
    }

    fn process_microphone(&mut self, samples: Vec<f32>, format: AudioFormat) -> Vec<f32> {
        if format != self.microphone_format {
            return samples;
        }
        self.microphone_remainder.extend(samples);
        self.process_available_microphone_frames()
    }

    fn process_system(&mut self, samples: Vec<f32>, format: AudioFormat) -> Vec<f32> {
        if format != self.system_format {
            return samples;
        }
        self.system_remainder.extend_from_slice(&samples);
        let frame_samples = stream_config(self.system_format).num_samples();

        while self.system_remainder.len() >= frame_samples {
            let frame: Vec<_> = self.system_remainder.drain(..frame_samples).collect();
            self.process_render_frame(frame);
        }

        samples
    }

    fn process_available_microphone_frames(&mut self) -> Vec<f32> {
        let frame_samples = stream_config(self.microphone_format).num_samples();
        let mut processed = Vec::new();

        while self.microphone_remainder.len() >= frame_samples {
            let frame: Vec<_> = self.microphone_remainder.drain(..frame_samples).collect();
            processed.extend(self.process_microphone_frame(frame));
        }

        processed
    }

    fn process_render_frame(&mut self, samples: Vec<f32>) {
        let input = deinterleave(&samples, self.system_format.channels);
        let mut output = empty_channels(stream_config(self.system_format));
        let input = input.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut output = output.iter_mut().map(Vec::as_mut_slice).collect::<Vec<_>>();

        self.processor
            .process_render_f32(&input, &mut output)
            .expect("validated system audio format should support echo cancellation");
    }

    fn process_microphone_frame(&mut self, samples: Vec<f32>) -> Vec<f32> {
        let input = deinterleave(&samples, self.microphone_format.channels);
        let mut output = empty_channels(stream_config(self.microphone_format));
        let input = input.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut output = output.iter_mut().map(Vec::as_mut_slice).collect::<Vec<_>>();

        self.processor
            .process_capture_f32(&input, &mut output)
            .expect("validated microphone format should support echo cancellation");
        interleave(&output)
    }

    fn flush_microphone(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.microphone_remainder)
    }
}

fn stream_config(format: AudioFormat) -> StreamConfig {
    StreamConfig::new(format.sample_rate, format.channels)
}

fn deinterleave(samples: &[f32], channels: u16) -> Vec<Vec<f32>> {
    let mut result =
        vec![Vec::with_capacity(samples.len() / usize::from(channels)); usize::from(channels)];

    for frame in samples.chunks_exact(usize::from(channels)) {
        for (channel, sample) in result.iter_mut().zip(frame) {
            channel.push(*sample);
        }
    }

    result
}

fn empty_channels(config: StreamConfig) -> Vec<Vec<f32>> {
    (0..config.num_channels())
        .map(|_| vec![0.0; config.num_frames()])
        .collect()
}

fn interleave(channels: &[&mut [f32]]) -> Vec<f32> {
    let frame_count = channels.first().map_or(0, |channel| channel.len());
    let mut samples = Vec::with_capacity(frame_count * channels.len());

    for frame in 0..frame_count {
        for channel in channels {
            samples.push(channel[frame]);
        }
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::{AudioFormat, AudioProcessor, EchoCancellation};

    const MICROPHONE_FORMAT: AudioFormat = AudioFormat {
        channels: 1,
        sample_rate: 48_000,
    };
    const SYSTEM_FORMAT: AudioFormat = AudioFormat {
        channels: 2,
        sample_rate: 48_000,
    };

    #[test]
    fn keeps_system_audio_unchanged_and_processes_microphone_audio() {
        let cancellation = EchoCancellation::new(MICROPHONE_FORMAT, SYSTEM_FORMAT);
        let microphone = AudioProcessor::microphone(cancellation.clone());
        let system = AudioProcessor::system(cancellation);
        let reference = (0..480)
            .flat_map(|index| {
                let sample = (index as f32 / 480.0 * std::f32::consts::TAU).sin() * 0.5;
                [sample, sample]
            })
            .collect::<Vec<_>>();
        let microphone_input = reference.iter().step_by(2).copied().collect::<Vec<_>>();

        assert_eq!(system.process(reference.clone(), SYSTEM_FORMAT), reference);
        let processed = microphone.process(microphone_input.clone(), MICROPHONE_FORMAT);

        assert_eq!(processed.len(), microphone_input.len());
        assert_ne!(processed, microphone_input);
    }

    #[test]
    fn flushes_partial_microphone_frames_without_losing_samples() {
        let cancellation = EchoCancellation::new(MICROPHONE_FORMAT, SYSTEM_FORMAT);
        let microphone = AudioProcessor::microphone(cancellation);
        let input = vec![0.25; 500];

        let processed = microphone.process(input.clone(), MICROPHONE_FORMAT);
        let remainder = microphone.flush();

        assert_eq!(processed.len() + remainder.len(), input.len());
        assert_eq!(remainder, vec![0.25; 20]);
    }

    #[test]
    fn supports_microphones_with_a_different_sample_rate_than_system_output() {
        let microphone_format = AudioFormat {
            channels: 1,
            sample_rate: 44_100,
        };
        let cancellation = EchoCancellation::new(microphone_format, SYSTEM_FORMAT);
        let microphone = AudioProcessor::microphone(cancellation.clone());
        let system = AudioProcessor::system(cancellation);

        system.process(vec![0.25; 960], SYSTEM_FORMAT);
        let processed = microphone.process(vec![0.25; 441], microphone_format);

        assert_eq!(processed.len(), 441);
    }

    #[test]
    fn preserves_audio_when_a_capture_format_changes() {
        let cancellation = EchoCancellation::new(MICROPHONE_FORMAT, SYSTEM_FORMAT);
        let microphone = AudioProcessor::microphone(cancellation);
        let input = vec![0.25; 441];

        assert_eq!(
            microphone.process(
                input.clone(),
                AudioFormat {
                    channels: 1,
                    sample_rate: 44_100,
                },
            ),
            input
        );
    }
}
