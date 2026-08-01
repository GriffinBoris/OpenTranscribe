use std::fs;
use std::path::Path;

use opentranscribe_domain::{
    Artifact, ArtifactKind, AudioSource, CaptureDevice, Codec, JobState, RecoveryState,
    SCHEMA_VERSION, SearchFilters, SessionLifecycle, SessionSource, Speaker, SpeakerSource,
    Transcript, TranscriptSegment, new_id, now,
};
use tempfile::tempdir;

use crate::audio::RecordingCapture;
use crate::error::AppError;

use super::{LibraryRepository, read_directories, read_files, relative_path};
use crate::storage::JobRequest;

mod imports_and_trash;
mod library_and_jobs;
mod moves;
mod recordings;
mod search_and_documents;
mod transcripts;

fn write_recovery_chunk(path: &Path, channels: u16, sample_rate: u32, samples: usize) {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("chunk should be created");

    for _ in 0..samples {
        writer
            .write_sample(0_i32)
            .expect("sample should be written");
    }

    writer.finalize().expect("chunk should finalize");
}
