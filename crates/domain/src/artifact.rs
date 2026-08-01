use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Microphone,
    System,
    Mixed,
    ImportedOriginal,
    ImportedAudio,
    Waveform,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AudioSource {
    Microphone,
    System,
    Mixed,
    Imported,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Codec {
    WavPcm24,
    Flac24,
    AacLc,
    Original,
    Json,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Artifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub relative_path: String,
    pub codec: Codec,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u16>,
    #[ts(type = "number | null")]
    pub duration_ms: Option<u64>,
    #[ts(type = "number")]
    pub byte_count: u64,
    pub sha256: String,
}
