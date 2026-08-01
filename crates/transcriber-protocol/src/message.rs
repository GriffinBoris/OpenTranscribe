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
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Command {
    Hello,
    LoadModel(ModelDescriptor),
    TranscribeFile(FileTranscription),
    UnloadModel,
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Event {
    Ready {
        runtime_version: String,
    },
    Segment {
        job_id: String,
        start_ms: u64,
        end_ms: u64,
        text: String,
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
