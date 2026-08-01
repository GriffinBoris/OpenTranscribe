use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Artifact, SCHEMA_VERSION};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum SessionSource {
    Recording,
    Import,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum SessionLifecycle {
    Draft,
    Recording,
    Paused,
    Finalizing,
    Ready,
    NeedsAttention,
    Recovered,
    Trashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryState {
    None,
    Recoverable,
    Recovered,
    Incomplete,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CaptureDevice {
    pub stable_id: String,
    pub label: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct PauseEvent {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    #[ts(type = "number")]
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct GapEvent {
    pub source: crate::AudioSource,
    #[ts(type = "number")]
    pub start_ms: u64,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Session {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub project_id: Option<String>,
    #[ts(type = "number")]
    pub revision: u64,
    pub source: SessionSource,
    pub lifecycle: SessionLifecycle,
    pub recovery_state: RecoveryState,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub microphone: Option<CaptureDevice>,
    pub system_output: Option<CaptureDevice>,
    pub artifacts: Vec<Artifact>,
    pub pauses: Vec<PauseEvent>,
    pub gaps: Vec<GapEvent>,
    pub language_hint: Option<String>,
    pub glossary: Vec<String>,
    pub openai_profile_id: Option<String>,
    pub current_transcript_id: Option<String>,
    pub transcript_run_ids: Vec<String>,
}

impl Session {
    pub fn new(title: String, project_id: Option<String>, source: SessionSource) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: crate::new_id(),
            title,
            project_id,
            revision: 1,
            source,
            lifecycle: SessionLifecycle::Draft,
            recovery_state: RecoveryState::None,
            created_at: crate::now(),
            started_at: None,
            stopped_at: None,
            duration_ms: 0,
            microphone: None,
            system_output: None,
            artifacts: Vec::new(),
            pauses: Vec::new(),
            gaps: Vec::new(),
            language_hint: None,
            glossary: Vec::new(),
            openai_profile_id: None,
            current_transcript_id: None,
            transcript_run_ids: Vec::new(),
        }
    }
}
