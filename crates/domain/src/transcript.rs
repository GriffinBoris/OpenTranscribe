use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AudioSource;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerSource {
    Microphone,
    System,
    Mixed,
    Diarized,
    Manual,
    Imported,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Speaker {
    pub id: String,
    pub display_name: String,
    pub source: SpeakerSource,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TranscriptSegment {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub speaker_id: String,
    pub source: AudioSource,
    pub edited: bool,
    pub word_timings: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Transcript {
    pub schema_version: u32,
    pub id: String,
    pub session_id: String,
    pub revision: u64,
    pub source_run_id: String,
    pub detected_language: Option<String>,
    pub language_hint: Option<String>,
    pub speakers: Vec<Speaker>,
    pub segments: Vec<TranscriptSegment>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptSource {
    Local,
    OpenAi,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptRunStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Canceled,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TranscriptRun {
    pub schema_version: u32,
    pub id: String,
    pub session_id: String,
    pub source: TranscriptSource,
    pub model_id: String,
    pub status: TranscriptRunStatus,
    pub input_artifact_ids: Vec<String>,
    pub language_hint: Option<String>,
    pub glossary: Vec<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub usage: Option<serde_json::Value>,
    pub approximate_cost_usd: Option<f64>,
    pub promoted_at: Option<DateTime<Utc>>,
}
