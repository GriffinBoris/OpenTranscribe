use super::{
    CLEANUP_MODEL, DictationConfiguration, DictationRun, delivery, history, panel, publish_status,
    transcription,
};
use crate::audio::StartRecordingOptions;
use crate::error::{AppError, AppResult};
use crate::local_models::installed_path;
use crate::state::AppState;
use opentranscribe_domain::{DictationPhase, DictationProvider, DictationStatus};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::Manager;
pub(super) fn start(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    if state.recorder.status().is_some() {
        return Err(AppError::RecordingActive);
    }
    let settings = state
        .settings
        .lock()
        .expect("app settings lock poisoned")
        .clone();
    let configuration = DictationConfiguration {
        provider: settings.dictation_provider,
        local_model_id: settings.dictation_local_model_id,
        openai_model: settings.dictation_openai_model,
        cleanup: settings.dictation_cleanup_enabled,
        auto_paste: settings.dictation_auto_paste,
        target: delivery::foreground_target(),
    };
    validate_configuration(&app, state, &configuration)?;
    let id = opentranscribe_domain::new_id();
    let directory = history::recordings_directory(&app)?.join(&id);
    std::fs::create_dir_all(&directory)?;
    if let Err(error) = state.recorder.start(
        StartRecordingOptions {
            session_id: id.clone(),
            microphone_device_id: settings.microphone_device_id,
            capture_system_audio: false,
            microphone_echo_cancellation: false,
            microphone_live_audio: None,
            system_live_audio: None,
        },
        directory.clone(),
    ) {
        history::remove_recovery_directory(&directory, "failed");
        return Err(error);
    }
    if let Err(error) = panel::show(&app) {
        let _ = state.recorder.stop();
        history::remove_recovery_directory(&directory, "failed");
        return Err(error);
    }
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    if let Some(previous) = run.recovery_directory.take() {
        history::remove_recovery_directory(&previous, "replaced");
    }
    *run = DictationRun {
        status: DictationStatus {
            id: Some(id.clone()),
            phase: DictationPhase::Recording,
            provider: Some(configuration.provider.clone()),
            ..DictationStatus::default()
        },
        recovery_directory: Some(directory),
        configuration: Some(configuration.clone()),
        ..DictationRun::default()
    };
    let canceled = run.canceled.clone();
    let status = run.status.clone();
    drop(run);
    publish_status(state);
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = transcription::prepare(&handle, &configuration, &canceled).await
            && !canceled.load(Ordering::SeqCst)
        {
            log::warn!("dictation prewarm failed: {error}");
        }
    });
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            let state = app.state::<AppState>();
            let status = super::status(&state);
            if status.id.as_deref() != Some(&id) || status.phase != DictationPhase::Recording {
                break;
            }
            publish_status(&state);
        }
    });
    Ok(status)
}

fn validate_configuration(
    app: &tauri::AppHandle,
    state: &AppState,
    configuration: &DictationConfiguration,
) -> AppResult<()> {
    if configuration.provider == DictationProvider::Local {
        let model = configuration.local_model_id.as_deref().ok_or_else(|| {
            AppError::Model("Choose an installed speech model in Settings → Dictation.".to_owned())
        })?;
        if model == CLEANUP_MODEL {
            return Err(AppError::Model(
                "S1-mini cleans text after speech recognition. Choose a speech model.".to_owned(),
            ));
        }
        installed_path(app, model)?;
    } else {
        state.openai_credentials.read()?;
    }
    if configuration.cleanup {
        installed_path(app, CLEANUP_MODEL)?;
    }
    Ok(())
}
