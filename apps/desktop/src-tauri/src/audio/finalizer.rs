use std::fs;
use std::path::{Path, PathBuf};

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use crate::audio::sync_and_rename;
use crate::error::{AppError, AppResult};

pub struct FinalizedTrack {
    pub path: PathBuf,
    pub recovery_chunks: Vec<PathBuf>,
}

pub fn finalize_microphone_track(
    recovery_directory: &Path,
    output_path: &Path,
) -> AppResult<FinalizedTrack> {
    finalize_track(recovery_directory, output_path, "microphone")
}

pub fn finalize_system_track(
    recovery_directory: &Path,
    output_path: &Path,
) -> AppResult<FinalizedTrack> {
    finalize_track(recovery_directory, output_path, "system")
}

fn finalize_track(
    recovery_directory: &Path,
    output_path: &Path,
    prefix: &str,
) -> AppResult<FinalizedTrack> {
    let recovery_chunks = recovery_chunks(recovery_directory, prefix)?;

    if recovery_chunks.is_empty() {
        return Err(AppError::Audio(format!(
            "the recording did not produce any {prefix} audio"
        )));
    }

    let first_reader = WavReader::open(&recovery_chunks[0]).map_err(audio_error)?;
    let spec = first_reader.spec();
    validate_spec(spec, prefix)?;
    drop(first_reader);
    let mut frame_count = 0_u64;

    for chunk_path in &recovery_chunks {
        let reader = WavReader::open(chunk_path).map_err(audio_error)?;
        ensure_matching_spec(spec, reader.spec(), prefix)?;
        frame_count += u64::from(reader.duration());
    }

    if output_path.exists() {
        validate_finalized_track(output_path, spec, frame_count, prefix)?;
        return Ok(FinalizedTrack {
            path: output_path.to_owned(),
            recovery_chunks,
        });
    }

    let temporary_path = output_path.with_extension("partial.wav");

    if temporary_path.exists() {
        fs::remove_file(&temporary_path)?;
    }

    let mut writer = WavWriter::create(&temporary_path, spec).map_err(audio_error)?;

    for chunk_path in &recovery_chunks {
        let mut reader = WavReader::open(chunk_path).map_err(audio_error)?;

        for sample in reader.samples::<i32>() {
            writer
                .write_sample(sample.map_err(audio_error)?)
                .map_err(audio_error)?;
        }
    }

    writer.finalize().map_err(audio_error)?;
    validate_finalized_track(&temporary_path, spec, frame_count, prefix)?;
    sync_and_rename(&temporary_path, output_path)?;

    Ok(FinalizedTrack {
        path: output_path.to_owned(),
        recovery_chunks,
    })
}

fn recovery_chunks(directory: &Path, prefix: &str) -> AppResult<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let expected_prefix = format!("{prefix}-");

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();

        if entry.file_type()?.is_file()
            && name.to_string_lossy().starts_with(&expected_prefix)
            && path.extension().is_some_and(|extension| extension == "wav")
        {
            paths.push(path);
        }
    }

    paths.sort();
    Ok(paths)
}

fn validate_spec(spec: WavSpec, prefix: &str) -> AppResult<()> {
    if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 24 {
        return Err(AppError::Audio(format!(
            "{prefix} recovery chunks must contain 24-bit PCM audio"
        )));
    }

    Ok(())
}

fn ensure_matching_spec(expected: WavSpec, actual: WavSpec, prefix: &str) -> AppResult<()> {
    if expected.channels != actual.channels
        || expected.sample_rate != actual.sample_rate
        || expected.bits_per_sample != actual.bits_per_sample
        || expected.sample_format != actual.sample_format
    {
        return Err(AppError::Audio(format!(
            "{prefix} recovery chunks use different audio formats"
        )));
    }

    Ok(())
}

fn validate_finalized_track(
    path: &Path,
    expected_spec: WavSpec,
    expected_frames: u64,
    prefix: &str,
) -> AppResult<()> {
    let reader = WavReader::open(path).map_err(audio_error)?;
    ensure_matching_spec(expected_spec, reader.spec(), prefix)?;

    if u64::from(reader.duration()) != expected_frames {
        return Err(AppError::Audio(format!(
            "the finalized {prefix} track failed duration validation"
        )));
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

    use super::{finalize_microphone_track, finalize_system_track};

    #[test]
    fn combines_recovery_chunks_into_one_playable_track() {
        let directory = tempdir().expect("temporary directory should exist");
        let recovery_directory = directory.path().join(".recovery");
        std::fs::create_dir(&recovery_directory).expect("recovery directory should exist");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 10,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        };

        write_chunk(&recovery_directory.join("microphone-000001.wav"), spec, 100);
        write_chunk(&recovery_directory.join("microphone-000002.wav"), spec, 50);

        let output_path = directory.path().join("microphone.wav");
        let track = finalize_microphone_track(&recovery_directory, &output_path)
            .expect("track should finalize");
        let reader = hound::WavReader::open(&track.path).expect("track should be readable");

        assert_eq!(reader.duration(), 150);
        assert_eq!(track.recovery_chunks.len(), 2);
    }

    #[test]
    fn combines_system_recovery_chunks() {
        let directory = tempdir().expect("temporary directory should exist");
        let recovery_directory = directory.path().join(".recovery");
        std::fs::create_dir(&recovery_directory).expect("recovery directory should exist");
        let spec = WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        };
        write_chunk(&recovery_directory.join("system-000001.wav"), spec, 200);

        let output_path = directory.path().join("system.wav");
        let track = finalize_system_track(&recovery_directory, &output_path)
            .expect("track should finalize");
        let reader = hound::WavReader::open(&track.path).expect("track should be readable");

        assert_eq!(reader.duration(), 100);
        assert_eq!(track.recovery_chunks.len(), 1);
    }

    #[test]
    fn reuses_a_valid_finalized_track_during_recovery() {
        let directory = tempdir().expect("temporary directory should exist");
        let recovery_directory = directory.path().join(".recovery");
        std::fs::create_dir(&recovery_directory).expect("recovery directory should exist");
        let spec = WavSpec {
            channels: 1,
            sample_rate: 10,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        };
        write_chunk(&recovery_directory.join("microphone-000001.wav"), spec, 10);

        let output_path = directory.path().join("microphone.wav");
        finalize_microphone_track(&recovery_directory, &output_path)
            .expect("first finalize should succeed");
        let track = finalize_microphone_track(&recovery_directory, &output_path)
            .expect("recovery finalize should be idempotent");

        assert_eq!(track.recovery_chunks.len(), 1);
    }

    fn write_chunk(path: &std::path::Path, spec: WavSpec, sample_count: usize) {
        let mut writer = WavWriter::create(path, spec).expect("chunk should be created");

        for _ in 0..sample_count {
            writer
                .write_sample(0_i32)
                .expect("sample should be written");
        }

        writer.finalize().expect("chunk should finalize");
    }
}
