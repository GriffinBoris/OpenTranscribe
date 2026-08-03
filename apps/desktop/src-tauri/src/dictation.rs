use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use opentranscribe_domain::{
    DictationHistoryEntry, DictationPhase, DictationProvider, DictationStatus,
    OpenAiTranscriptionModel,
};
use tauri::{Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use crate::audio::{StartRecordingOptions, finalize_microphone_track};
use crate::error::{AppError, AppResult};
use crate::local_models::{installed_path, transcribe_file};
use crate::state::AppState;
use crate::storage::atomic_file;
use crate::transcription::{
    OpenAiFileTranscriber, OpenAiTranscriptionRequest, estimate_openai_cost, openai_usage,
};

const PANEL_LABEL: &str = "dictation";
const PANEL_WIDTH: u32 = 520;
const PANEL_HEIGHT: u32 = 180;
const STATUS_UPDATE_INTERVAL: Duration = Duration::from_millis(50);
const HISTORY_FILENAME: &str = "history.json";

#[derive(Default)]
pub(crate) struct DictationRun {
    pub(crate) status: DictationStatus,
    recovery_directory: Option<PathBuf>,
    configuration: Option<DictationConfiguration>,
}

#[derive(Clone)]
struct DictationConfiguration {
    provider: DictationProvider,
    local_model_id: Option<String>,
    openai_model: OpenAiTranscriptionModel,
    auto_paste: bool,
}

struct DictationTranscript {
    text: String,
    provider: DictationProvider,
    model_id: String,
    usage: Option<serde_json::Value>,
    approximate_cost_usd: Option<f64>,
}

pub(crate) fn toggle(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    let phase = state
        .dictation
        .lock()
        .expect("dictation lock poisoned")
        .status
        .phase
        .clone();

    match phase {
        DictationPhase::Recording => stop(app, state),
        DictationPhase::Transcribing => Err(AppError::Application(
            "wait for dictation to finish before starting another one".to_owned(),
        )),
        DictationPhase::Idle | DictationPhase::Completed | DictationPhase::Failed => {
            start(app, state)
        }
    }
}

pub(crate) fn is_active(state: &AppState) -> bool {
    matches!(
        state
            .dictation
            .lock()
            .expect("dictation lock poisoned")
            .status
            .phase,
        DictationPhase::Recording | DictationPhase::Transcribing
    )
}

pub(crate) fn status(state: &AppState) -> DictationStatus {
    let mut status = state
        .dictation
        .lock()
        .expect("dictation lock poisoned")
        .status
        .clone();

    if status.phase == DictationPhase::Recording
        && let Some(recording_status) = state.recorder.status()
        && status.id.as_deref() == Some(recording_status.session_id.as_str())
    {
        status.elapsed_ms = recording_status.elapsed_ms;
        status.microphone_peak = recording_status.microphone_peak;
    }

    status
}

pub(crate) fn history(app: &tauri::AppHandle) -> AppResult<Vec<DictationHistoryEntry>> {
    let path = history_path(app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn clear_history(app: &tauri::AppHandle) -> AppResult<()> {
    clear_history_file(&history_path(app)?)
}

fn clear_history_file(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

pub(crate) fn dismiss(app: &tauri::AppHandle) {
    hide_panel(app);
}

pub(crate) fn cancel(app: &tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    let (id, recovery_directory) = {
        let run = state.dictation.lock().expect("dictation lock poisoned");

        if run.status.phase != DictationPhase::Recording {
            return Ok(run.status.clone());
        }

        (
            run.status.id.clone().ok_or(AppError::RecordingInactive)?,
            run.recovery_directory
                .clone()
                .ok_or(AppError::RecordingInactive)?,
        )
    };
    let recorder_status = state.recorder.stop()?;

    if recorder_status.session_id != id {
        return Err(AppError::RecordingInactive);
    }

    let status = DictationStatus::default();
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    run.status = status.clone();
    run.recovery_directory = None;
    run.configuration = None;
    drop(run);

    remove_recovery_directory(&recovery_directory, "canceled");
    hide_panel(app);
    publish_status(state);
    Ok(status)
}

fn start(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
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
        auto_paste: settings.dictation_auto_paste,
    };

    if configuration.provider == DictationProvider::Local {
        let model_id = configuration.local_model_id.as_deref().ok_or_else(|| {
            AppError::Model("choose an installed local model for dictation in Settings".to_owned())
        })?;
        installed_path(&app, model_id)?;
    }

    let id = opentranscribe_domain::new_id();
    let recovery_directory = recordings_directory(&app)?.join(&id);
    fs::create_dir_all(&recovery_directory)?;

    let options = StartRecordingOptions {
        session_id: id.clone(),
        microphone_device_id: settings.microphone_device_id,
        capture_system_audio: false,
        microphone_live_audio: None,
        system_live_audio: None,
    };

    if let Err(error) = state.recorder.start(options, recovery_directory.clone()) {
        remove_recovery_directory(&recovery_directory, "failed");
        return Err(error);
    }

    if let Err(error) = show_panel(&app) {
        let _ = state.recorder.stop();
        remove_recovery_directory(&recovery_directory, "failed");
        return Err(error);
    }

    let status = DictationStatus {
        id: Some(id.clone()),
        phase: DictationPhase::Recording,
        provider: Some(configuration.provider.clone()),
        text: None,
        error_message: None,
        elapsed_ms: 0,
        microphone_peak: 0.0,
        auto_pasted: false,
        approximate_cost_usd: None,
    };
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    run.status = status.clone();
    run.recovery_directory = Some(recovery_directory);
    run.configuration = Some(configuration);
    drop(run);
    publish_status(state);
    publish_recording_statuses(app, id);
    Ok(status)
}

fn stop(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let recorder_status = state.recorder.stop()?;
    let mut run = state.dictation.lock().expect("dictation lock poisoned");

    if run.status.id.as_deref() != Some(recorder_status.session_id.as_str()) {
        return Err(AppError::RecordingInactive);
    }

    let recovery_directory = run
        .recovery_directory
        .clone()
        .ok_or(AppError::RecordingInactive)?;
    let configuration = run
        .configuration
        .clone()
        .ok_or(AppError::RecordingInactive)?;
    run.status.phase = DictationPhase::Transcribing;
    run.status.elapsed_ms = recorder_status.elapsed_ms;
    run.status.microphone_peak = 0.0;
    let status = run.status.clone();
    drop(run);
    publish_status(state);

    tauri::async_runtime::spawn(transcribe(
        app,
        recorder_status.session_id,
        recovery_directory,
        configuration,
    ));
    Ok(status)
}

async fn transcribe(
    app: tauri::AppHandle,
    id: String,
    recovery_directory: PathBuf,
    configuration: DictationConfiguration,
) {
    let audio_path = recovery_directory.join("dictation.wav");
    let result = async {
        finalize_microphone_track(&recovery_directory, &audio_path)?;
        let duration_ms = audio_duration_ms(&audio_path)?;
        let transcript = match configuration.provider {
            DictationProvider::Local => {
                let model_id = configuration.local_model_id.ok_or_else(|| {
                    AppError::Model(
                        "choose an installed local model for dictation in Settings".to_owned(),
                    )
                })?;
                let text =
                    transcribe_file(&app, model_id.clone(), audio_path.clone(), |_, _| {}).await?;
                DictationTranscript {
                    text,
                    provider: DictationProvider::Local,
                    model_id,
                    usage: None,
                    approximate_cost_usd: None,
                }
            }
            DictationProvider::OpenAi => {
                let api_key = app.state::<AppState>().openai_credentials.read()?;
                let model_id = configuration.openai_model.model_id().to_owned();
                let request_path = audio_path.clone();
                let request_model_id = model_id.clone();
                let transcripts = tauri::async_runtime::spawn_blocking(move || {
                    OpenAiFileTranscriber::new().transcribe(
                        OpenAiTranscriptionRequest {
                            api_key: &api_key,
                            model: &request_model_id,
                            files: &[request_path],
                            language_hint: None,
                        },
                        |_, _| {},
                        |_| {},
                        || false,
                    )
                })
                .await
                .map_err(|error| AppError::Provider(error.to_string()))??;
                let text = transcripts
                    .iter()
                    .map(|transcript| transcript.text.trim())
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                let usage = openai_usage(
                    &model_id,
                    duration_ms,
                    0,
                    transcripts
                        .iter()
                        .filter_map(|transcript| transcript.usage.clone())
                        .collect(),
                );
                let approximate_cost_usd = estimate_openai_cost(&model_id, duration_ms, 0);
                DictationTranscript {
                    text,
                    provider: DictationProvider::OpenAi,
                    model_id,
                    usage: Some(usage),
                    approximate_cost_usd,
                }
            }
        };

        if transcript.text.is_empty() {
            return Err(AppError::Application(
                "the dictation did not contain any speech".to_owned(),
            ));
        }

        save_history(
            &app,
            DictationHistoryEntry {
                id: id.clone(),
                text: transcript.text.clone(),
                created_at: opentranscribe_domain::now(),
                provider: transcript.provider.clone(),
                model_id: transcript.model_id,
                usage: transcript.usage,
                approximate_cost_usd: transcript.approximate_cost_usd,
            },
        )?;

        hide_panel(&app);
        tokio::time::sleep(Duration::from_millis(120)).await;
        let auto_pasted = deliver(&app, transcript.text.clone(), configuration.auto_paste).await?;
        Ok::<(String, bool, Option<f64>), AppError>((
            transcript.text,
            auto_pasted,
            transcript.approximate_cost_usd,
        ))
    }
    .await;

    let state = app.state::<AppState>();
    let mut run = state.dictation.lock().expect("dictation lock poisoned");

    if run.status.id.as_deref() != Some(id.as_str()) {
        return;
    }

    match result {
        Ok((text, auto_pasted, approximate_cost_usd)) => {
            run.status.phase = DictationPhase::Completed;
            run.status.text = Some(text);
            run.status.error_message = None;
            run.status.auto_pasted = auto_pasted;
            run.status.approximate_cost_usd = approximate_cost_usd;
            run.recovery_directory = None;
            run.configuration = None;
            remove_recovery_directory(&recovery_directory, "completed");
        }
        Err(error) => {
            run.status.phase = DictationPhase::Failed;
            run.status.error_message = Some(error.to_string());
            run.recovery_directory = None;
            run.configuration = None;
            remove_recovery_directory(&recovery_directory, "failed");
        }
    }
    drop(run);
    publish_status(&state);
}

fn audio_duration_ms(path: &std::path::Path) -> AppResult<u64> {
    let reader =
        hound::WavReader::open(path).map_err(|error| AppError::Audio(error.to_string()))?;
    Ok(u64::from(reader.duration()) * 1_000 / u64::from(reader.spec().sample_rate))
}

fn save_history(app: &tauri::AppHandle, entry: DictationHistoryEntry) -> AppResult<()> {
    let path = history_path(app)?;
    let mut entries = history(app)?;
    entries.insert(0, entry);
    atomic_file::write_json(&path, &entries)
}

fn history_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("dictation");
    fs::create_dir_all(&directory)?;
    Ok(directory.join(HISTORY_FILENAME))
}

fn recordings_directory(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("dictation");
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn remove_recovery_directory(path: &Path, outcome: &str) {
    if let Err(error) = fs::remove_dir_all(path) {
        log::warn!("could not remove {outcome} dictation audio: {error}");
    }
}

async fn deliver(app: &tauri::AppHandle, text: String, auto_paste: bool) -> AppResult<bool> {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    app.run_on_main_thread(move || {
        let _ = sender.send(deliver_on_main_thread(&text, auto_paste));
    })
    .map_err(|error| AppError::Application(error.to_string()))?;

    receiver
        .await
        .map_err(|_| AppError::Application("dictation delivery was interrupted".to_owned()))?
}

fn deliver_on_main_thread(text: &str, auto_paste: bool) -> AppResult<bool> {
    let mut clipboard =
        Clipboard::new().map_err(|error| AppError::Application(error.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|error| AppError::Application(error.to_string()))?;

    if !auto_paste {
        return Ok(false);
    }

    let mut input = Enigo::new(&Settings::default())
        .map_err(|error| AppError::Application(error.to_string()))?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    if let Err(error) = input
        .key(modifier, Direction::Press)
        .and_then(|_| input.key(Key::Unicode('v'), Direction::Click))
        .and_then(|_| input.key(modifier, Direction::Release))
    {
        log::warn!("dictation was copied but could not be pasted automatically: {error}");
        return Ok(false);
    }
    Ok(true)
}

fn show_panel(app: &tauri::AppHandle) -> AppResult<()> {
    let window = match app.get_webview_window(PANEL_LABEL) {
        Some(window) => window,
        None => WebviewWindowBuilder::new(
            app,
            PANEL_LABEL,
            WebviewUrl::App("index.html?dictation=1".into()),
        )
        .title("Dictation")
        .inner_size(f64::from(PANEL_WIDTH), f64::from(PANEL_HEIGHT))
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .focusable(true)
        .visible(false)
        .build()
        .map_err(|error| AppError::Application(error.to_string()))?,
    };
    position_panel(app, &window)?;
    if let Some(main_window) = app.get_webview_window("main") {
        main_window
            .hide()
            .map_err(|error| AppError::Application(error.to_string()))?;
    }
    window
        .show()
        .map_err(|error| AppError::Application(error.to_string()))
}

fn position_panel(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> AppResult<()> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Application("the main window is unavailable".to_owned()))?;
    let monitor = main_window
        .current_monitor()
        .map_err(|error| AppError::Application(error.to_string()))?
        .or_else(|| main_window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let position = monitor.position();
    let size = monitor.size();
    let panel_size = window
        .outer_size()
        .map_err(|error| AppError::Application(error.to_string()))?;
    let x = position.x + ((size.width.saturating_sub(panel_size.width)) / 2) as i32;
    let y = position.y + ((size.height.saturating_sub(panel_size.height)) / 2) as i32;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| AppError::Application(error.to_string()))
}

fn hide_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(PANEL_LABEL)
        && let Err(error) = window.hide()
    {
        log::warn!("could not hide dictation panel: {error}");
    }
}

fn publish_status(state: &AppState) {
    let status = status(state);
    let channel = state
        .dictation_status_channel
        .lock()
        .expect("dictation status channel lock poisoned")
        .clone();
    let Some(channel) = channel else {
        return;
    };

    if let Err(error) = channel.send(status) {
        log::warn!("dictation status subscriber disconnected: {error}");
    }
}

fn publish_recording_statuses(app: tauri::AppHandle, id: String) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(STATUS_UPDATE_INTERVAL);

        loop {
            interval.tick().await;

            let state = app.state::<AppState>();
            let is_current_recording = {
                let run = state.dictation.lock().expect("dictation lock poisoned");
                run.status.phase == DictationPhase::Recording
                    && run.status.id.as_deref() == Some(id.as_str())
            };

            if !is_current_recording {
                return;
            }

            publish_status(&state);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::fs;

    use opentranscribe_domain::DictationPhase;
    use tempfile::tempdir;

    use crate::state::AppState;

    use super::{clear_history_file, is_active, remove_recovery_directory};

    #[test]
    fn treats_recording_and_transcribing_dictation_as_active_work() {
        let state = AppState::default();

        assert!(!is_active(&state));

        state
            .dictation
            .lock()
            .expect("dictation lock should be available")
            .status
            .phase = DictationPhase::Recording;
        assert!(is_active(&state));

        state
            .dictation
            .lock()
            .expect("dictation lock should be available")
            .status
            .phase = DictationPhase::Transcribing;
        assert!(is_active(&state));

        state
            .dictation
            .lock()
            .expect("dictation lock should be available")
            .status
            .phase = DictationPhase::Completed;
        assert!(!is_active(&state));
    }

    #[test]
    fn removes_recovery_audio_after_a_failed_dictation() {
        let temporary_directory = tempdir().expect("temporary directory should be created");
        let recovery_directory = temporary_directory.path().join("dictation");
        fs::create_dir_all(&recovery_directory).expect("recovery directory should be created");
        fs::write(recovery_directory.join("dictation.wav"), "audio")
            .expect("recovery audio should be created");

        remove_recovery_directory(&recovery_directory, "failed");

        assert!(!recovery_directory.exists());
    }

    #[test]
    fn clears_dictation_history_without_removing_its_directory() {
        let temporary_directory = tempdir().expect("temporary directory should be created");
        let history_path = temporary_directory.path().join("history.json");
        fs::write(&history_path, "[]").expect("dictation history should be created");

        clear_history_file(&history_path).expect("dictation history should be cleared");

        assert!(!history_path.exists());
        assert!(temporary_directory.path().exists());
    }
}
