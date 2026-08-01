use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    FinalizeRecording,
    ImportMedia,
    TranscribeLocal,
    TranscribeOpenAi,
    DiarizeOpenAi,
    DownloadModel,
    Export,
    MoveLibrary,
    ReindexLibrary,
    DownloadUpdate,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    BlockedOffline,
    Preparing,
    Running,
    AwaitingUser,
    Completed,
    Failed,
    Canceled,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum JobStage {
    Preparing,
    Downloading,
    Decoding,
    Uploading,
    Transcribing,
    Diarizing,
    Finalizing,
    Indexing,
    Exporting,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ProgressUnit {
    AudioMs,
    Bytes,
    Items,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct JobProgress {
    pub stage: JobStage,
    #[ts(type = "number")]
    pub completed_units: u64,
    #[ts(type = "number | null")]
    pub total_units: Option<u64>,
    pub unit: ProgressUnit,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Job {
    pub id: String,
    pub session_id: Option<String>,
    pub kind: JobKind,
    pub state: JobState,
    pub progress: Option<JobProgress>,
    #[serde(default)]
    pub estimated_cost_usd: Option<f64>,
    pub attempt: u8,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    pub fn new(session_id: Option<String>, kind: JobKind) -> Self {
        let timestamp = crate::now();

        Self {
            id: crate::new_id(),
            session_id,
            kind,
            state: JobState::Queued,
            progress: None,
            estimated_cost_usd: None,
            attempt: 1,
            error_message: None,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}
