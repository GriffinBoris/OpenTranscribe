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
pub struct StreamDescriptor {
    pub stream_id: String,
    pub source: String,
    pub language_hint: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AudioChunk {
    pub stream_id: String,
    pub sequence: u64,
    pub start_ms: u64,
    pub pcm_s16le: Vec<u8>,
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
    StartStream(StreamDescriptor),
    AudioChunk(AudioChunk),
    EndStream { stream_id: String },
    TranscribeFile(FileTranscription),
    Cancel { job_id: String },
    UnloadModel,
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Event {
    Ready {
        runtime_version: String,
    },
    ModelProgress {
        completed: u64,
        total: u64,
    },
    ChunkAck {
        stream_id: String,
        sequence: u64,
    },
    Partial {
        stream_id: String,
        start_ms: u64,
        end_ms: u64,
        text: String,
    },
    Final {
        stream_id: String,
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
