use std::fs;
use std::path::Path;

use opentranscribe_domain::{Job, JobKind, JobState, RecordingMode};
use serde::{Deserialize, Serialize};

use super::atomic_file;
use crate::error::AppResult;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobRequest {
    FinalizeRecording {
        mode: RecordingMode,
        openai_model_id: String,
    },
    TranscribeLocal {
        model_id: String,
    },
    TranscribeOpenAi {
        model_id: String,
        #[serde(default)]
        live_stream_count: u8,
    },
}

impl JobRequest {
    pub fn job_kind(&self) -> JobKind {
        match self {
            Self::FinalizeRecording { .. } => JobKind::FinalizeRecording,
            Self::TranscribeLocal { .. } => JobKind::TranscribeLocal,
            Self::TranscribeOpenAi { .. } => JobKind::TranscribeOpenAi,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JobRecord {
    pub job: Job,
    pub request: JobRequest,
}

pub fn create(directory: &Path, session_id: String, request: JobRequest) -> AppResult<JobRecord> {
    let record = JobRecord {
        job: Job::new(Some(session_id), request.job_kind()),
        request,
    };
    save(directory, &record)?;
    Ok(record)
}

pub fn load(directory: &Path, job_id: &str) -> AppResult<JobRecord> {
    Ok(serde_json::from_slice(&fs::read(
        directory.join(format!("{job_id}.json")),
    )?)?)
}

pub fn list(directory: &Path) -> AppResult<Vec<JobRecord>> {
    let mut records = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
        {
            records.push(serde_json::from_slice(&fs::read(path)?)?);
        }
    }

    records.sort_by(|left: &JobRecord, right: &JobRecord| {
        right.job.created_at.cmp(&left.job.created_at)
    });
    Ok(records)
}

pub fn save(directory: &Path, record: &JobRecord) -> AppResult<()> {
    atomic_file::write_json(&directory.join(format!("{}.json", record.job.id)), record)
}

pub fn mark_interrupted(directory: &Path) -> AppResult<()> {
    for mut record in list(directory)? {
        if matches!(
            record.job.state,
            JobState::Queued | JobState::Preparing | JobState::Running
        ) {
            record.job.state = JobState::Failed;
            record.job.error_message = Some(
                "OpenTranscribe closed before this job finished. Retry it to continue.".to_owned(),
            );
            record.job.updated_at = opentranscribe_domain::now();
            save(directory, &record)?;
        }
    }

    Ok(())
}
