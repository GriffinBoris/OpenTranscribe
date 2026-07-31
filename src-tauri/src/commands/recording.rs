use opentranscribe_domain::{
    AppEvent, OpenAiTranscriptionModel, RecordingMode, Session, SessionLifecycle,
};
use serde::Deserialize;

use super::events::publish_recording_levels;
use super::{ActiveRecordingIntent, AppState, send_event, with_repository};
use crate::audio::{AudioDevices, RecordingStatus, StartRecordingOptions};
use crate::credentials::OpenAiCredentials;
use crate::error::{AppError, AppResult};
use crate::jobs;
use crate::transcription::OpenAiRealtimeController;

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
    let realtime_api_key = if request.recording_mode == RecordingMode::OpenAiLive {
        Some(OpenAiCredentials::read()?)
    } else {
        None
    };
    let session = with_repository(&state, |repository| {
        repository.create_recording_session(
            request.title,
            request.project_id,
            request.language_hint.clone(),
        )
    })?;
    let openai_model = request.openai_model.unwrap_or_else(|| {
        state
            .settings
            .lock()
            .expect("app state lock poisoned")
            .openai_transcription_model
            .clone()
    });
    let live_transcription = if request.recording_mode == RecordingMode::OpenAiLive {
        Some(OpenAiRealtimeController::start(
            app.clone(),
            session.id.clone(),
            realtime_api_key.expect("OpenAI live transcription requires a credential"),
            request.language_hint.clone(),
            request.capture_system_audio,
        ))
    } else {
        None
    };
    let options = StartRecordingOptions {
        session_id: session.id.clone(),
        microphone_device_id: request.microphone_device_id,
        capture_system_audio: request.capture_system_audio,
        microphone_live_audio: live_transcription
            .as_ref()
            .map(OpenAiRealtimeController::microphone_sink),
        system_live_audio: live_transcription
            .as_ref()
            .and_then(OpenAiRealtimeController::system_sink),
    };
    let recovery_directory = with_repository(&state, |repository| {
        repository.recording_directory(&options.session_id)
    })?;
    let capture = match state.recorder.start(options.clone(), recovery_directory) {
        Ok(capture) => capture,
        Err(error) => {
            if let Some(live_transcription) = live_transcription {
                live_transcription.stop();
            }
            discard_failed_recording_session(&state, &session.id);
            return Err(error);
        }
    };
    let session = match with_repository(&state, |repository| {
        repository.begin_recording(&options.session_id, capture)
    }) {
        Ok(session) => session,
        Err(error) => {
            let _ = state.recorder.stop();
            if let Some(live_transcription) = live_transcription {
                live_transcription.stop();
            }
            discard_failed_recording_session(&state, &session.id);
            return Err(error);
        }
    };
    *state
        .active_recording_intent
        .lock()
        .expect("app state lock poisoned") = Some(ActiveRecordingIntent {
        session_id: session.id.clone(),
        mode: request.recording_mode,
        openai_model,
    });
    *state
        .live_transcription
        .lock()
        .expect("app state lock poisoned") = live_transcription;

    send_event(
        &state,
        AppEvent::RecordingStateChanged(SessionLifecycle::Recording),
    );
    crate::tray::set_recording_state(&app, Some(false));
    publish_recording_levels(app, options.session_id);
    Ok(session)
}

#[tauri::command]
pub fn pause_recording(
    paused: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<RecordingStatus> {
    let status = state.recorder.pause(paused)?;
    send_event(
        &state,
        AppEvent::RecordingStateChanged(if paused {
            SessionLifecycle::Paused
        } else {
            SessionLifecycle::Recording
        }),
    );
    crate::tray::set_recording_state(&app, Some(paused));
    Ok(status)
}

#[tauri::command]
pub fn recording_status(state: tauri::State<'_, AppState>) -> Option<RecordingStatus> {
    state.recorder.status()
}

#[tauri::command]
pub fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Session> {
    send_event(
        &state,
        AppEvent::RecordingStateChanged(SessionLifecycle::Finalizing),
    );
    let status = state.recorder.stop()?;
    let live_transcription = state
        .live_transcription
        .lock()
        .expect("app state lock poisoned")
        .take();
    let session = match with_repository(&state, |repository| {
        repository.finish_recording(&status.session_id)
    }) {
        Ok(session) => session,
        Err(error) => {
            if let Some(live_transcription) = live_transcription {
                live_transcription.stop();
            }
            return Err(handle_finalization_failure(
                &app,
                &state,
                &status.session_id,
                error,
            ));
        }
    };
    if let Some(live_transcription) = live_transcription {
        live_transcription.stop();
    }
    send_event(
        &state,
        AppEvent::RecordingStateChanged(SessionLifecycle::Ready),
    );
    send_event(&state, AppEvent::LibraryChanged);
    crate::tray::set_recording_state(&app, None);
    enqueue_post_recording_transcription(&app, &state, &session.id);
    Ok(session)
}

fn handle_finalization_failure(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, AppState>,
    session_id: &str,
    error: AppError,
) -> AppError {
    let message = format!(
        "Recording stopped and its recovery audio was preserved, but finalization needs attention: {error}"
    );
    log::error!("{message}");

    if let Err(mark_error) = with_repository(state, |repository| {
        repository.mark_recording_needs_attention(session_id)
    }) {
        log::error!("failed to mark the recording for recovery: {mark_error}");
    }

    state
        .active_recording_intent
        .lock()
        .expect("app state lock poisoned")
        .take();
    if let Some(live_transcription) = state
        .live_transcription
        .lock()
        .expect("app state lock poisoned")
        .take()
    {
        live_transcription.stop();
    }
    crate::tray::set_recording_state(app, None);
    send_event(
        state,
        AppEvent::RecordingStateChanged(SessionLifecycle::NeedsAttention),
    );
    send_event(state, AppEvent::LibraryChanged);
    send_event(state, AppEvent::AttentionRequired(message.clone()));
    AppError::Application(message)
}

fn enqueue_post_recording_transcription(
    app: &tauri::AppHandle,
    state: &tauri::State<'_, AppState>,
    session_id: &str,
) {
    let intent = state
        .active_recording_intent
        .lock()
        .expect("app state lock poisoned")
        .take();
    let Some(intent) = intent else {
        let message =
            "Recording saved, but its automatic transcription preference was unavailable."
                .to_owned();
        log::warn!("{message}");
        send_event(state, AppEvent::AttentionRequired(message));
        return;
    };

    if intent.session_id != session_id {
        let message = "Recording saved, but its automatic transcription preference did not match the completed session.".to_owned();
        log::warn!("{message}");
        send_event(state, AppEvent::AttentionRequired(message));
        return;
    }

    if let Err(error) = jobs::enqueue_post_recording_transcription(
        app,
        session_id,
        intent.mode,
        intent.openai_model.model_id().to_owned(),
    ) {
        let message =
            format!("Recording saved, but automatic transcription could not start: {error}");
        log::warn!("{message}");
        send_event(state, AppEvent::AttentionRequired(message));
    }
}

fn discard_failed_recording_session(state: &tauri::State<'_, AppState>, session_id: &str) {
    if let Err(error) = with_repository(state, |repository| {
        repository.discard_draft_session(session_id)
    }) {
        log::warn!("failed to roll back a recording session that never started: {error}");
    }
}
