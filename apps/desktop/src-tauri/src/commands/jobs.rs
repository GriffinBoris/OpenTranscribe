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
    pub diarization_model_id: Option<String>,
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
            diarization_model_id: request.diarization_model_id,
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

#[derive(Deserialize)]
pub struct EnqueueDiarizationRequest {
    pub session_id: String,
    pub model_id: String,
    pub transcript_id: String,
    pub expected_revision: u64,
}

#[tauri::command]
pub fn enqueue_diarization(
    request: EnqueueDiarizationRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    jobs::diarization::enqueue(
        jobs::diarization::DiarizationRequest {
            session_id: request.session_id,
            model_id: request.model_id,
            transcript_id: request.transcript_id,
            expected_revision: request.expected_revision,
        },
        app,
        &state,
    )
}
