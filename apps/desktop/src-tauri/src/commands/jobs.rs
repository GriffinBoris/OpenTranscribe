use opentranscribe_domain::Job;
use serde::Deserialize;

use super::AppState;
use crate::error::AppResult;
use crate::jobs::{self, TranscriptionProvider, TranscriptionRequest};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionProviderRequest {
    Local,
    OpenAi,
}

#[derive(Deserialize)]
pub struct EnqueueTranscriptionRequest {
    pub session_id: String,
    pub provider: TranscriptionProviderRequest,
    pub model_id: String,
}

impl From<EnqueueTranscriptionRequest> for TranscriptionRequest {
    fn from(request: EnqueueTranscriptionRequest) -> Self {
        Self {
            session_id: request.session_id,
            provider: match request.provider {
                TranscriptionProviderRequest::Local => TranscriptionProvider::Local,
                TranscriptionProviderRequest::OpenAi => TranscriptionProvider::OpenAi,
            },
            model_id: request.model_id,
        }
    }
}

#[tauri::command]
pub fn enqueue_transcription(
    request: EnqueueTranscriptionRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    jobs::enqueue_transcription(request.into(), false, app, &state)
}

#[tauri::command]
pub fn retry_job(
    job_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    jobs::retry_job(&job_id, app, &state)
}

#[tauri::command]
pub fn cancel_job(job_id: String, state: tauri::State<'_, AppState>) -> AppResult<Job> {
    jobs::cancel_job(&job_id, &state)
}
