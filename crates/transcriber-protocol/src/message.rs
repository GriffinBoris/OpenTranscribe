use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Envelope<T> {
    pub protocol_version: u16,
    pub request_id: String,
    pub body: T,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ModelDescriptor {
    pub model_id: String,
    pub path: String,
    pub use_gpu: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileTranscription {
    pub job_id: String,
    pub path: String,
    pub language_hint: Option<String>,
    pub prompt: Option<String>,
    pub diarization_model_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileDiarization {
    pub job_id: String,
    pub path: String,
    pub model_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpeakerTurn {
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Command {
    Hello,
    LoadModel(ModelDescriptor),
    TranscribeFile(FileTranscription),
    DiarizeFile(FileDiarization),
    NormalizeText { job_id: String, text: String },
    UnloadModel,
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Event {
    NormalizedText {
        job_id: String,
        text: String,
    },
    Ready {
        runtime_version: String,
    },
    Segment {
        job_id: String,
        start_ms: u64,
        end_ms: u64,
        text: String,
        speaker_label: Option<String>,
    },
    SpeakerTurn {
        job_id: String,
        turn: SpeakerTurn,
    },
    JobProgress {
        job_id: String,
        completed_ms: u64,
        total_ms: u64,
    },
    Completed {
        job_id: String,
    },
    Error {
        code: String,
        message: String,
    },
    Status {
        model_id: Option<String>,
        backlog_ms: u64,
    },
}
