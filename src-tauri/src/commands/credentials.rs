use serde::{Deserialize, Serialize};

use crate::credentials::{CredentialStatus, OpenAiCredentials};
use crate::error::AppResult;
use crate::transcription::OpenAiFileTranscriber;

#[derive(Deserialize)]
pub struct SaveOpenAiKeyRequest {
    pub api_key: String,
}

#[derive(Serialize)]
pub struct ConnectionTestResult {
    pub connected: bool,
    pub message: String,
}

#[tauri::command]
pub fn openai_credential_status() -> AppResult<CredentialStatus> {
    OpenAiCredentials::status()
}

#[tauri::command]
pub fn save_openai_api_key(request: SaveOpenAiKeyRequest) -> AppResult<CredentialStatus> {
    OpenAiCredentials::save(&request.api_key)
}

#[tauri::command]
pub fn remove_openai_api_key() -> AppResult<CredentialStatus> {
    OpenAiCredentials::remove()
}

#[tauri::command]
pub fn test_openai_connection() -> AppResult<ConnectionTestResult> {
    let api_key = OpenAiCredentials::read()?;
    OpenAiFileTranscriber::new().test_connection(&api_key)?;
    Ok(ConnectionTestResult {
        connected: true,
        message: "OpenAI API connection is ready".to_owned(),
    })
}
