use opentranscribe_domain::{OpenAiTranscriptionModel, RecordingMode, Session};
use serde::Deserialize;

use super::AppState;
use crate::audio::{AudioDevices, RecordingStatus};
use crate::error::{AppError, AppResult};
use crate::recording::{self, RecordingRequest};

#[derive(Deserialize)]
pub struct CreateRecordingRequest {
    pub title: String,
    pub project_id: Option<String>,
    pub microphone_device_id: Option<String>,
    pub capture_system_audio: bool,
    pub language_hint: Option<String>,
    pub recording_mode: RecordingMode,
    pub openai_model: Option<OpenAiTranscriptionModel>,
}

impl From<CreateRecordingRequest> for RecordingRequest {
    fn from(request: CreateRecordingRequest) -> Self {
        Self {
            title: request.title,
            project_id: request.project_id,
            microphone_device_id: request.microphone_device_id,
            capture_system_audio: request.capture_system_audio,
            language_hint: request.language_hint,
            recording_mode: request.recording_mode,
            openai_model: request.openai_model,
        }
    }
}

#[tauri::command]
pub fn audio_devices(state: tauri::State<'_, AppState>) -> AppResult<AudioDevices> {
    state.recorder.devices()
}

#[tauri::command]
pub fn open_system_audio_permission_settings() -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        tauri_plugin_opener::open_url(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
            None::<&str>,
        )
        .map_err(|error| AppError::Application(error.to_string()))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(AppError::Application(
            "system audio permission settings are only available on macOS".to_owned(),
        ))
    }
}

#[tauri::command]
pub fn create_recording(
    request: CreateRecordingRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    recording::start(request.into(), app, &state)
}

#[tauri::command]
pub fn pause_recording(
    paused: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<RecordingStatus> {
    recording::pause(paused, &app, &state)
}

#[tauri::command]
pub fn recording_status(state: tauri::State<'_, AppState>) -> Option<RecordingStatus> {
    recording::status(&state)
}

#[tauri::command]
pub fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    recording::stop(&app, &state)
}
