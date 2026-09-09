use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::DictationProvider;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum DictationPhase {
    #[default]
    Idle,
    Recording,
    Transcribing,
    Cleaning,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DictationStatus {
    pub id: Option<String>,
    pub phase: DictationPhase,
    pub provider: Option<DictationProvider>,
    pub text: Option<String>,
    pub error_message: Option<String>,
    #[ts(type = "number")]
    pub elapsed_ms: u64,
    pub microphone_peak: f32,
    pub auto_pasted: bool,
    pub can_retry: bool,
    pub progress_percent: Option<u8>,
    pub approximate_cost_usd: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DictationHistoryEntry {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub raw_text: Option<String>,
    #[serde(default)]
    pub cleanup_model_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub provider: DictationProvider,
    pub model_id: String,
    pub usage: Option<serde_json::Value>,
    pub approximate_cost_usd: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::DictationHistoryEntry;

    #[test]
    fn reads_history_created_before_optional_cleanup() {
        let entry: DictationHistoryEntry = serde_json::from_str(
            r#"{
            "id":"previous", "text":"Original wording.", "created_at":"2026-09-01T12:00:00Z",
            "provider":"local", "model_id":"whisper-small-q5_1", "usage":null,
            "approximate_cost_usd":null
        }"#,
        )
        .expect("previous history remains readable");
        assert_eq!(entry.text, "Original wording.");
        assert_eq!(entry.raw_text, None);
        assert_eq!(entry.cleanup_model_id, None);
    }
}
