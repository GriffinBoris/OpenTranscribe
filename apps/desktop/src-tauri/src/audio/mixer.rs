use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use crate::error::{AppError, AppResult};

const OUTPUT_CHANNELS: u16 = 2;
const OUTPUT_SAMPLE_RATE: u32 = 48_000;
const PCM_24_MAX: f32 = 8_388_607.0;

pub fn mix_tracks(
    microphone_path: &Path,
    system_path: &Path,
    output_path: &Path,
) -> AppResult<PathBuf> {
    let mut microphone = TrackReader::open(microphone_path)?;
    let mut system = TrackReader::open(system_path)?;
    let output_frames = microphone.output_frames.max(system.output_frames);

    if output_path.exists() {
        validate_mix(output_path, output_frames)?;
        return Ok(output_path.to_owned());
    }

    let temporary_path = output_path.with_extension("partial.wav");

    if temporary_path.exists() {
        fs::remove_file(&temporary_path)?;
    }

    let mut writer = WavWriter::create(
        &temporary_path,
        WavSpec {
            channels: OUTPUT_CHANNELS,
            sample_rate: OUTPUT_SAMPLE_RATE,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        },
    )
    .map_err(audio_error)?;

    for _ in 0..output_frames {
        let microphone_frame = microphone.next_output_frame()?;
        let system_frame = system.next_output_frame()?;

        for channel in 0..usize::from(OUTPUT_CHANNELS) {
            let sample =
                (microphone_frame[channel] * 0.5 + system_frame[channel] * 0.5).clamp(-1.0, 1.0);
            writer
                .write_sample((sample * PCM_24_MAX).round() as i32)
                .map_err(audio_error)?;
        }
    }

    writer.finalize().map_err(audio_error)?;
    fs::File::open(&temporary_path)?.sync_all()?;
    validate_mix(&temporary_path, output_frames)?;
    fs::rename(&temporary_path, output_path)?;
    Ok(output_path.to_owned())
}

struct TrackReader {
    reader: WavReader<BufReader<fs::File>>,
    channels: u16,
    sample_step: f64,
    phase: f64,
    current: Option<[f32; 2]>,
    next: Option<[f32; 2]>,
    output_frames: u64,
    emitted_frames: u64,
}

impl TrackReader {
    fn open(path: &Path) -> AppResult<Self> {
        let mut reader = WavReader::open(path).map_err(audio_error)?;
        let spec = reader.spec();

        if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 24 {
            return Err(AppError::Audio(
                "tracks must contain 24-bit PCM audio before mixing".to_owned(),
            ));
        }

        let input_frames = u64::from(reader.duration());
        let output_frames =
            (input_frames * u64::from(OUTPUT_SAMPLE_RATE)).div_ceil(u64::from(spec.sample_rate));
        let current = read_frame(&mut reader, spec.channels)?;
        let next = read_frame(&mut reader, spec.channels)?;

        Ok(Self {
            reader,
            channels: spec.channels,
            sample_step: f64::from(spec.sample_rate) / f64::from(OUTPUT_SAMPLE_RATE),
            phase: 0.0,
            current,
            next,
            output_frames,
            emitted_frames: 0,
        })
    }

    fn next_output_frame(&mut self) -> AppResult<[f32; 2]> {
        if self.emitted_frames >= self.output_frames {
            return Ok([0.0; 2]);
        }

        let current = self.current.unwrap_or([0.0; 2]);
        let next = self.next.unwrap_or(current);
        let fraction = self.phase as f32;
        let frame = [
            current[0] + (next[0] - current[0]) * fraction,
            current[1] + (next[1] - current[1]) * fraction,
        ];
        self.phase += self.sample_step;

        while self.phase >= 1.0 {
            self.current = self.next;
            self.next = read_frame(&mut self.reader, self.channels)?;
            self.phase -= 1.0;
        }

        self.emitted_frames += 1;
        Ok(frame)
    }
}

fn read_frame(
    reader: &mut WavReader<BufReader<fs::File>>,
    channels: u16,
) -> AppResult<Option<[f32; 2]>> {
    let mut samples = Vec::with_capacity(usize::from(channels));

    for channel in 0..channels {
        let Some(sample) = reader.samples::<i32>().next() else {
            if channel == 0 {
                return Ok(None);
            }

            return Err(AppError::Audio(
                "an audio track ended in the middle of a frame".to_owned(),
            ));
        };
        samples.push(sample.map_err(audio_error)? as f32 / PCM_24_MAX);
    }

    let frame = if channels == 1 {
        [samples[0]; 2]
    } else {
        let mut sums = [0.0_f32; 2];
        let mut counts = [0_u16; 2];

        for (index, sample) in samples.into_iter().enumerate() {
            let output_channel = index % 2;
            sums[output_channel] += sample;
            counts[output_channel] += 1;
        }

        [
            sums[0] / f32::from(counts[0]),
            sums[1] / f32::from(counts[1]),
        ]
    };
    Ok(Some(frame))
}

fn validate_mix(path: &Path, expected_frames: u64) -> AppResult<()> {
    let reader = WavReader::open(path).map_err(audio_error)?;
    let spec = reader.spec();

    if spec.channels != OUTPUT_CHANNELS
        || spec.sample_rate != OUTPUT_SAMPLE_RATE
        || spec.bits_per_sample != 24
        || spec.sample_format != SampleFormat::Int
        || u64::from(reader.duration()) != expected_frames
    {
        return Err(AppError::Audio(
            "the mixed track failed audio validation".to_owned(),
        ));
    }

    Ok(())
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

#[cfg(test)]
mod tests {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use tempfile::tempdir;

    use super::mix_tracks;

    #[test]
    fn mixes_and_resamples_tracks_to_stereo_48khz() {
        let directory = tempdir().expect("temporary directory should exist");
        let microphone_path = directory.path().join("microphone.wav");
        let system_path = directory.path().join("system.wav");
        write_track(&microphone_path, 1, 24_000, 24, 4_194_304);
        write_track(&system_path, 2, 48_000, 96, 2_097_152);

        let output_path = directory.path().join("mixed.wav");
        mix_tracks(&microphone_path, &system_path, &output_path).expect("tracks should be mixed");
        let mut reader = hound::WavReader::open(output_path).expect("mix should be readable");
        let spec = reader.spec();
        let duration = reader.duration();
        let samples = reader
            .samples::<i32>()
            .collect::<Result<Vec<_>, _>>()
            .expect("mix samples should be readable");

        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 48_000);
        assert_eq!(duration, 48);
        assert!(samples.iter().all(|sample| *sample > 0));
    }

    #[test]
    fn reuses_a_valid_existing_mix() {
        let directory = tempdir().expect("temporary directory should exist");
        let microphone_path = directory.path().join("microphone.wav");
        let system_path = directory.path().join("system.wav");
        let output_path = directory.path().join("mixed.wav");
        write_track(&microphone_path, 1, 48_000, 48, 1);
        write_track(&system_path, 2, 48_000, 96, 1);

        mix_tracks(&microphone_path, &system_path, &output_path).expect("first mix should succeed");
        mix_tracks(&microphone_path, &system_path, &output_path)
            .expect("valid mix should be reusable");
    }

    fn write_track(
        path: &std::path::Path,
        channels: u16,
        sample_rate: u32,
        sample_count: usize,
        sample: i32,
    ) {
        let mut writer = WavWriter::create(
            path,
            WavSpec {
                channels,
                sample_rate,
                bits_per_sample: 24,
                sample_format: SampleFormat::Int,
            },
        )
        .expect("track should be created");

        for _ in 0..sample_count {
            writer
                .write_sample(sample)
                .expect("sample should be written");
        }

        writer.finalize().expect("track should finalize");
    }
}
