use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AudioSource, Job, JobProgress, SessionLifecycle};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct LevelSnapshot {
    pub session_id: String,
    pub is_paused: bool,
    pub captures_system_audio: bool,
    pub microphone_peak: f32,
    pub system_peak: f32,
    #[ts(type = "number")]
    pub elapsed_ms: u64,
    #[ts(type = "number")]
    pub dropped_packets: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct LiveTranscriptUpdate {
    pub session_id: String,
    pub item_id: String,
    pub source: AudioSource,
    pub text: String,
    pub completed: bool,
    #[ts(type = "number")]
    pub started_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TranscriptionPreviewUpdate {
    pub session_id: String,
    pub job_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum AppEvent {
    RecordingStateChanged(SessionLifecycle),
    RecordingLevels(LevelSnapshot),
    LiveTranscriptChanged(LiveTranscriptUpdate),
    TranscriptionPreviewChanged(TranscriptionPreviewUpdate),
    JobProgress {
        job_id: String,
        progress: JobProgress,
    },
    JobStateChanged(Job),
    DictationStateChanged(crate::DictationStatus),
    LibraryChanged,
    ImportRequested,
    AttentionRequired(String),
}
