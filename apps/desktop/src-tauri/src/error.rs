use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("database failed: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("a library must be selected first")]
    LibraryNotSelected,
    #[error("the requested item does not exist")]
    NotFound,
    #[error("the file changed outside OpenTranscribe")]
    RevisionConflict,
    #[error("the requested name is empty or reserved")]
    InvalidName,
    #[error("audio capture failed: {0}")]
    Audio(String),
    #[error("a recording is already active")]
    RecordingActive,
    #[error("there is no active recording")]
    RecordingInactive,
    #[error("credential storage failed: {0}")]
    Credential(String),
    #[error("provider request failed: {0}")]
    Provider(String),
    #[error("local model failed: {0}")]
    Model(String),
    #[error("export failed: {0}")]
    Export(String),
    #[error("the job was canceled")]
    JobCanceled,
    #[error("{0}")]
    Application(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
