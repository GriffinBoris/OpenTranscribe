use std::fs;
use std::io::BufWriter;
use std::path::Path;

use hound::{SampleFormat, WavSpec, WavWriter};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::error::{AppError, AppResult};

const OUTPUT_CHANNELS: u16 = 2;
const OUTPUT_SAMPLE_RATE: u32 = 48_000;
const PCM_24_MAX: f32 = 8_388_607.0;

pub struct ImportedAudio {
    pub duration_ms: u64,
}

pub fn extract_audio(source_path: &Path, output_path: &Path) -> AppResult<ImportedAudio> {
    let source = fs::File::open(source_path)?;
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();

    if let Some(extension) = source_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        hint.with_extension(extension);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(audio_error)?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| AppError::Audio("the file does not contain an audio track".to_owned()))?;
    let track_id = track.id;
    let codec_parameters = track
        .codec_params
        .as_ref()
        .and_then(|parameters| parameters.audio())
        .ok_or_else(|| AppError::Audio("the audio track has no codec parameters".to_owned()))?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(codec_parameters, &AudioDecoderOptions::default())
        .map_err(audio_error)?;
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
    let mut source_sample_rate = None;
    let mut source_channel_count = None;
    let mut resampler = None;

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(error) => return Err(audio_error(error)),
        };

        if packet.track_id != track_id {
            continue;
        }

        let audio = decoder.decode(&packet).map_err(audio_error)?;
        let sample_rate = audio.spec().rate();
        let channel_count = audio.spec().channels().count();

        if source_sample_rate.is_none() {
            source_sample_rate = Some(sample_rate);
            source_channel_count = Some(channel_count);
            resampler = Some(StereoResampler::new(sample_rate));
        }

        if source_sample_rate != Some(sample_rate) || source_channel_count != Some(channel_count) {
            return Err(AppError::Audio(
                "audio format changes within the imported file are not supported".to_owned(),
            ));
        }

        let mut samples = vec![0.0_f32; audio.samples_interleaved()];
        audio.copy_to_slice_interleaved(&mut samples);

        for frame in samples.chunks_exact(channel_count) {
            resampler
                .as_mut()
                .expect("audio resampler must be initialized")
                .push(stereo_frame(frame), &mut writer)?;
        }
    }

    let resampler = resampler
        .as_mut()
        .ok_or_else(|| AppError::Audio("the imported audio track is empty".to_owned()))?;
    resampler.finish(&mut writer)?;
    writer.finalize().map_err(audio_error)?;
    fs::File::open(&temporary_path)?.sync_all()?;
    fs::rename(&temporary_path, output_path)?;

    Ok(ImportedAudio {
        duration_ms: resampler.output_frames * 1_000 / u64::from(OUTPUT_SAMPLE_RATE),
    })
}

struct StereoResampler {
    previous: Option<[f32; 2]>,
    input_frames: u64,
    output_frames: u64,
    next_output_position: f64,
    source_frames_per_output: f64,
}

impl StereoResampler {
    fn new(source_sample_rate: u32) -> Self {
        Self {
            previous: None,
            input_frames: 0,
            output_frames: 0,
            next_output_position: 0.0,
            source_frames_per_output: f64::from(source_sample_rate) / f64::from(OUTPUT_SAMPLE_RATE),
        }
    }

    fn push(
        &mut self,
        frame: [f32; 2],
        writer: &mut WavWriter<BufWriter<fs::File>>,
    ) -> AppResult<()> {
        let Some(previous) = self.previous else {
            self.previous = Some(frame);
            self.input_frames = 1;
            return Ok(());
        };
        let current_position = self.input_frames as f64;

        while self.next_output_position < current_position {
            let fraction =
                (self.next_output_position - (current_position - 1.0)).clamp(0.0, 1.0) as f32;
            self.write(
                [
                    previous[0] + (frame[0] - previous[0]) * fraction,
                    previous[1] + (frame[1] - previous[1]) * fraction,
                ],
                writer,
            )?;
            self.next_output_position += self.source_frames_per_output;
        }

        self.previous = Some(frame);
        self.input_frames += 1;
        Ok(())
    }

    fn finish(&mut self, writer: &mut WavWriter<BufWriter<fs::File>>) -> AppResult<()> {
        let frame = self
            .previous
            .ok_or_else(|| AppError::Audio("the imported audio track is empty".to_owned()))?;

        while self.next_output_position < self.input_frames as f64 {
            self.write(frame, writer)?;
            self.next_output_position += self.source_frames_per_output;
        }

        Ok(())
    }

    fn write(
        &mut self,
        frame: [f32; 2],
        writer: &mut WavWriter<BufWriter<fs::File>>,
    ) -> AppResult<()> {
        for sample in frame {
            writer
                .write_sample((sample.clamp(-1.0, 1.0) * PCM_24_MAX).round() as i32)
                .map_err(audio_error)?;
        }
        self.output_frames += 1;
        Ok(())
    }
}

fn stereo_frame(frame: &[f32]) -> [f32; 2] {
    if frame.len() == 1 {
        return [frame[0]; 2];
    }

    let mut sums = [0.0_f32; 2];
    let mut counts = [0_u16; 2];

    for (index, sample) in frame.iter().enumerate() {
        let output_channel = index % 2;
        sums[output_channel] += sample;
        counts[output_channel] += 1;
    }

    [
        sums[0] / f32::from(counts[0]),
        sums[1] / f32::from(counts[1]),
    ]
}

fn audio_error(error: impl std::fmt::Display) -> AppError {
    AppError::Audio(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{StereoResampler, stereo_frame};

    #[test]
    fn downmixes_imported_audio_to_stereo() {
        assert_eq!(stereo_frame(&[1.0]), [1.0, 1.0]);
        assert_eq!(stereo_frame(&[1.0, -1.0, 0.5, 0.5]), [0.75, -0.25]);
    }

    #[test]
    fn configures_resampling_against_the_output_rate() {
        let resampler = StereoResampler::new(24_000);
        assert_eq!(resampler.source_frames_per_output, 0.5);
    }
}
