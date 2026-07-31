use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

const WHISPER_SAMPLE_RATE: u32 = 16_000;

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub duration_ms: u64,
}

pub fn decode_for_whisper(path: &Path) -> Result<DecodedAudio, String> {
    let source = File::open(path).map_err(|error| error.to_string())?;
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();

    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }

    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|error| error.to_string())?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| "the file does not contain an audio track".to_owned())?;
    let track_id = track.id;
    let codec_parameters = track
        .codec_params
        .as_ref()
        .and_then(|parameters| parameters.audio())
        .ok_or_else(|| "the audio track has no codec parameters".to_owned())?;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(codec_parameters, &AudioDecoderOptions::default())
        .map_err(|error| error.to_string())?;
    let mut interleaved = Vec::new();
    let mut sample_rate = None;
    let mut channel_count = None;

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(error) => return Err(error.to_string()),
        };

        if packet.track_id != track_id {
            continue;
        }

        let audio = decoder.decode(&packet).map_err(|error| match error {
            SymphoniaError::DecodeError(message) => message.to_owned(),
            error => error.to_string(),
        })?;
        let packet_sample_rate = audio.spec().rate();
        let packet_channel_count = audio.spec().channels().count();

        if sample_rate.is_none() {
            sample_rate = Some(packet_sample_rate);
            channel_count = Some(packet_channel_count);
        }

        if sample_rate != Some(packet_sample_rate) || channel_count != Some(packet_channel_count) {
            return Err("audio format changes within the file are not supported".to_owned());
        }

        let start = interleaved.len();
        interleaved.resize(start + audio.samples_interleaved(), 0.0);
        audio.copy_to_slice_interleaved(&mut interleaved[start..]);
    }

    let sample_rate = sample_rate.ok_or_else(|| "the audio track is empty".to_owned())?;
    let channel_count =
        channel_count.ok_or_else(|| "the audio track has no channel information".to_owned())?;
    let mono = downmix(&interleaved, channel_count);

    if mono.is_empty() {
        return Err("the audio track is empty".to_owned());
    }

    let samples = resample(&mono, sample_rate, WHISPER_SAMPLE_RATE);
    let duration_ms = samples.len() as u64 * 1_000 / u64::from(WHISPER_SAMPLE_RATE);

    Ok(DecodedAudio {
        samples,
        duration_ms,
    })
}

fn downmix(interleaved: &[f32], channel_count: usize) -> Vec<f32> {
    interleaved
        .chunks_exact(channel_count)
        .map(|frame| frame.iter().sum::<f32>() / channel_count as f32)
        .collect()
}

fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }

    let output_length = samples.len() * target_rate as usize / source_rate as usize;
    let source_per_output = source_rate as f64 / target_rate as f64;

    (0..output_length)
        .map(|index| {
            let source_position = index as f64 * source_per_output;
            let left_index = source_position.floor() as usize;
            let right_index = (left_index + 1).min(samples.len() - 1);
            let fraction = (source_position - left_index as f64) as f32;
            samples[left_index] * (1.0 - fraction) + samples[right_index] * fraction
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{downmix, resample};

    #[test]
    fn downmixes_stereo_frames() {
        assert_eq!(downmix(&[1.0, -1.0, 0.5, 0.5], 2), vec![0.0, 0.5]);
    }

    #[test]
    fn keeps_samples_when_rate_matches() {
        assert_eq!(resample(&[0.0, 1.0], 16_000, 16_000), vec![0.0, 1.0]);
    }
}
