use serde::{Deserialize, Serialize};

use super::AppState;
use crate::credentials::CredentialStatus;
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
pub fn openai_credential_status(state: tauri::State<'_, AppState>) -> AppResult<CredentialStatus> {
    state.openai_credentials.status()
}

#[tauri::command]
pub fn save_openai_api_key(
    request: SaveOpenAiKeyRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<CredentialStatus> {
    state.openai_credentials.save(&request.api_key)
}

#[tauri::command]
pub fn remove_openai_api_key(state: tauri::State<'_, AppState>) -> AppResult<CredentialStatus> {
    state.openai_credentials.remove()
}

#[tauri::command]
pub fn test_openai_connection(
    state: tauri::State<'_, AppState>,
) -> AppResult<ConnectionTestResult> {
    let api_key = state.openai_credentials.read()?;
    OpenAiFileTranscriber::new().test_connection(&api_key)?;
    Ok(ConnectionTestResult {
        connected: true,
        message: "OpenAI API connection is ready".to_owned(),
    })
}
