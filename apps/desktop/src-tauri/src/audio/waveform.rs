use std::path::Path;

use crate::error::{AppError, AppResult};

pub fn waveform_peaks(path: &Path, bucket_count: usize) -> AppResult<Vec<f32>> {
    let mut reader =
        hound::WavReader::open(path).map_err(|error| AppError::Audio(error.to_string()))?;
    let sample_count = reader.len() as usize;
    let mut peaks = vec![0.0_f32; bucket_count];

    if sample_count == 0 || bucket_count == 0 {
        return Ok(peaks);
    }

    for (sample_index, sample) in reader.samples::<i32>().enumerate() {
        let sample = sample.map_err(|error| AppError::Audio(error.to_string()))?;
        let bucket_index = (sample_index * bucket_count / sample_count).min(bucket_count - 1);
        let amplitude = sample.unsigned_abs() as f32 / 8_388_608.0;
        peaks[bucket_index] = peaks[bucket_index].max(amplitude.min(1.0));
    }

    Ok(peaks)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::waveform_peaks;

    #[test]
    fn creates_bounded_waveform_peaks() {
        let directory = tempdir().expect("temporary directory should exist");
        let path = directory.path().join("audio.wav");
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 8_000,
                bits_per_sample: 24,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .expect("wave file should be created");

        for sample in [0, 2_000_000, -4_000_000, 8_000_000] {
            writer.write_sample(sample).expect("sample should write");
        }
        writer.finalize().expect("wave file should finalize");

        let peaks = waveform_peaks(&path, 2).expect("waveform should be generated");

        assert_eq!(peaks.len(), 2);
        assert!(peaks[0] > 0.2);
        assert!(peaks[1] > 0.9);
        assert!(peaks.iter().all(|peak| *peak <= 1.0));
    }
}
