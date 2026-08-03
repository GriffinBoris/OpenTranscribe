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
    pub approximate_cost_usd: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct DictationHistoryEntry {
    pub id: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub provider: DictationProvider,
    pub model_id: String,
    pub usage: Option<serde_json::Value>,
    pub approximate_cost_usd: Option<f64>,
}
