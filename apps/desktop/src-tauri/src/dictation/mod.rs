mod capture;
mod delivery;
mod history;
mod panel;
mod providers;
mod transcription;
pub(crate) mod workers;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use opentranscribe_domain::{
    AppEvent, DictationPhase, DictationProvider, DictationStatus, OpenAiTranscriptionModel,
};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::Manager;

pub(crate) use history::{clear_history, clear_temporary_recordings, history};
pub(super) const CLEANUP_MODEL: &str = "s1-mini-q4_k_m";

#[derive(Default)]
pub(crate) struct DictationRun {
    pub(crate) status: DictationStatus,
    recovery_directory: Option<PathBuf>,
    configuration: Option<DictationConfiguration>,
    canceled: Arc<AtomicBool>,
    transcript: Option<transcription::Transcript>,
}

#[derive(Clone)]
struct DictationConfiguration {
    provider: DictationProvider,
    local_model_id: Option<String>,
    openai_model: OpenAiTranscriptionModel,
    cleanup: bool,
    auto_paste: bool,
    target: Option<u64>,
}

pub(crate) fn is_active(state: &AppState) -> bool {
    active_phase(
        &state
            .dictation
            .lock()
            .expect("dictation lock poisoned")
            .status
            .phase,
    )
}

fn active_phase(phase: &DictationPhase) -> bool {
    matches!(
        phase,
        DictationPhase::Recording | DictationPhase::Transcribing | DictationPhase::Cleaning
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
        && let Some(recording) = state.recorder.status()
        && status.id.as_deref() == Some(&recording.session_id)
    {
        status.elapsed_ms = recording.elapsed_ms;
        status.microphone_peak = recording.microphone_peak;
    }
    status
}

pub(crate) fn toggle(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    let phase = status(state).phase;
    if phase == DictationPhase::Recording {
        return stop(app, state);
    }
    if active_phase(&phase) {
        return Err(AppError::Application(
            "Cancel or wait for this dictation to finish.".to_owned(),
        ));
    }
    capture::start(app, state)
}

pub(crate) fn dismiss(app: &tauri::AppHandle) {
    if let Err(error) = cancel(app, &app.state::<AppState>()) {
        log::warn!("could not dismiss dictation: {error}");
    }
}

pub(crate) fn cancel(app: &tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    let phase = status(state).phase;
    if phase == DictationPhase::Recording {
        state.recorder.stop()?;
    }
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    run.canceled.store(true, Ordering::SeqCst);
    let directory = run.recovery_directory.take();
    *run = DictationRun::default();
    drop(run);
    // Inference owns its input until it acknowledges cancellation.
    if !matches!(
        phase,
        DictationPhase::Transcribing | DictationPhase::Cleaning
    ) && let Some(directory) = directory
    {
        history::remove_recovery_directory(&directory, "discarded");
    }
    panel::hide(app);
    publish_status(state);
    Ok(DictationStatus::default())
}

fn stop(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let recording = state.recorder.stop()?;
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    run.status.elapsed_ms = recording.elapsed_ms;
    run.status.microphone_peak = 0.0;
    launch(app, state, run)
}

pub(crate) fn retry(app: tauri::AppHandle, state: &AppState) -> AppResult<DictationStatus> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    let run = state.dictation.lock().expect("dictation lock poisoned");
    if !run.status.can_retry || active_phase(&run.status.phase) {
        return Err(AppError::Application(
            "There is no failed dictation to retry.".to_owned(),
        ));
    }
    launch(app, state, run)
}

fn launch(
    app: tauri::AppHandle,
    state: &AppState,
    mut run: std::sync::MutexGuard<'_, DictationRun>,
) -> AppResult<DictationStatus> {
    let id = run.status.id.clone().ok_or(AppError::RecordingInactive)?;
    let directory = run
        .recovery_directory
        .clone()
        .ok_or(AppError::RecordingInactive)?;
    let configuration = run
        .configuration
        .clone()
        .ok_or(AppError::RecordingInactive)?;
    run.status.phase = DictationPhase::Transcribing;
    run.status.can_retry = false;
    run.status.error_message = None;
    run.status.progress_percent = None;
    let canceled = run.canceled.clone();
    let transcript = run.transcript.clone();
    let status = run.status.clone();
    drop(run);
    publish_status(state);
    tauri::async_runtime::spawn(transcription::run(
        app,
        id,
        directory,
        configuration,
        canceled,
        transcript,
    ));
    Ok(status)
}

fn publish_status(state: &AppState) {
    let status = status(state);
    if let Some(channel) = state
        .dictation_status_channel
        .lock()
        .expect("dictation channel lock poisoned")
        .clone()
        && let Err(error) = channel.send(status.clone())
    {
        log::warn!("dictation status subscriber disconnected: {error}");
    }
    crate::events::send_event(state, AppEvent::DictationStateChanged(status));
}

#[cfg(test)]
mod tests {
    use super::active_phase;
    use opentranscribe_domain::DictationPhase;
    #[test]
    fn protects_every_in_flight_stage() {
        for phase in [
            DictationPhase::Recording,
            DictationPhase::Transcribing,
            DictationPhase::Cleaning,
        ] {
            assert!(active_phase(&phase));
        }
        for phase in [
            DictationPhase::Idle,
            DictationPhase::Completed,
            DictationPhase::Failed,
        ] {
            assert!(!active_phase(&phase));
        }
    }
}
