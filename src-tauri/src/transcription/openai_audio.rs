use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::error::{AppError, AppResult};

const MAX_FILE_BYTES: u64 = 25 * 1024 * 1024;
const TARGET_CHUNK_BYTES: u64 = 24 * 1024 * 1024;

pub struct PreparedAudio {
    _temporary_directory: TempDir,
    pub chunks: Vec<PreparedChunk>,
}

pub struct PreparedChunk {
    pub path: PathBuf,
    pub duration_ms: u64,
}

pub fn prepare_audio(files: &[PathBuf]) -> AppResult<PreparedAudio> {
    let temporary_directory = tempfile::tempdir()?;
    let mut chunks = Vec::new();

    for (source_index, path) in files.iter().enumerate() {
        if std::fs::metadata(path)?.len() <= MAX_FILE_BYTES {
            chunks.push(PreparedChunk {
                path: path.clone(),
                duration_ms: wav_duration_ms(path)?,
            });
            continue;
        }

        if path.extension().is_none_or(|extension| extension != "wav") {
            return Err(AppError::Provider(format!(
                "{} is larger than OpenAI's 25 MB limit and must be converted before upload",
                path.to_string_lossy()
            )));
        }

        chunks.extend(split_wav(
            path,
            temporary_directory.path(),
            source_index,
            TARGET_CHUNK_BYTES,
        )?);
    }

    Ok(PreparedAudio {
        _temporary_directory: temporary_directory,
        chunks,
    })
}

fn wav_duration_ms(path: &Path) -> AppResult<u64> {
    if path.extension().is_none_or(|extension| extension != "wav") {
        return Ok(0);
    }

    let reader = hound::WavReader::open(path)
        .map_err(|error| AppError::Provider(format!("audio file is not readable: {error}")))?;
    Ok(u64::from(reader.duration()) * 1_000 / u64::from(reader.spec().sample_rate))
}

fn split_wav(
    path: &Path,
    directory: &Path,
    source_index: usize,
    maximum_bytes: u64,
) -> AppResult<Vec<PreparedChunk>> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|error| AppError::Provider(format!("audio file is not readable: {error}")))?;
    let spec = reader.spec();

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 24 {
        return Err(AppError::Provider(
            "large WAV uploads must contain 24-bit PCM audio".to_owned(),
        ));
    }

    let bytes_per_frame = u64::from(spec.channels) * 3;
    let frames_per_chunk = maximum_bytes
        .saturating_sub(128)
        .checked_div(bytes_per_frame)
        .filter(|frame_count| *frame_count > 0)
        .ok_or_else(|| AppError::Provider("OpenAI audio chunk size is invalid".to_owned()))?;
    let samples_per_chunk = frames_per_chunk * u64::from(spec.channels);
    let mut chunks = Vec::new();
    let mut writer = None;
    let mut samples_written = 0_u64;
    let mut chunk_index = 0_usize;

    for sample in reader.samples::<i32>() {
        if writer.is_none() {
            chunk_index += 1;
            writer = Some(
                hound::WavWriter::create(chunk_path(directory, source_index, chunk_index), spec)
                    .map_err(|error| AppError::Provider(error.to_string()))?,
            );
        }

        writer
            .as_mut()
            .expect("writer is created before samples are written")
            .write_sample(sample.map_err(|error| AppError::Provider(error.to_string()))?)
            .map_err(|error| AppError::Provider(error.to_string()))?;
        samples_written += 1;

        if samples_written == samples_per_chunk {
            writer
                .take()
                .expect("writer exists when a chunk is complete")
                .finalize()
                .map_err(|error| AppError::Provider(error.to_string()))?;
            chunks.push(PreparedChunk {
                path: chunk_path(directory, source_index, chunk_index),
                duration_ms: frames_per_chunk * 1_000 / u64::from(spec.sample_rate),
            });
            samples_written = 0;
        }
    }

    if let Some(unfinished) = writer {
        unfinished
            .finalize()
            .map_err(|error| AppError::Provider(error.to_string()))?;
        let frame_count = samples_written / u64::from(spec.channels);
        chunks.push(PreparedChunk {
            path: chunk_path(directory, source_index, chunk_index),
            duration_ms: frame_count * 1_000 / u64::from(spec.sample_rate),
        });
    }

    Ok(chunks)
}

fn chunk_path(directory: &Path, source_index: usize, chunk_index: usize) -> PathBuf {
    directory.join(format!("openai-{source_index:03}-{chunk_index:04}.wav"))
}

#[cfg(test)]
mod tests {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use tempfile::tempdir;

    use super::split_wav;

    #[test]
    fn splits_large_recordings_into_provider_safe_wav_files() {
        let directory = tempdir().expect("temporary directory should exist");
        let source = directory.path().join("recording.wav");
        let mut writer = WavWriter::create(
            &source,
            WavSpec {
                channels: 1,
                sample_rate: 10,
                bits_per_sample: 24,
                sample_format: SampleFormat::Int,
            },
        )
        .expect("source should be created");

        for _ in 0..40 {
            writer
                .write_sample(0_i32)
                .expect("sample should be written");
        }

        writer.finalize().expect("source should finalize");
        let output = directory.path().join("chunks");
        std::fs::create_dir(&output).expect("chunk directory should exist");
        let chunks = split_wav(&source, &output, 0, 158).expect("source should split");

        assert_eq!(chunks.len(), 4);
        assert_eq!(
            chunks.iter().map(|chunk| chunk.duration_ms).sum::<u64>(),
            4_000
        );
        assert!(chunks.iter().all(|chunk| chunk.path.exists()));
    }
}
