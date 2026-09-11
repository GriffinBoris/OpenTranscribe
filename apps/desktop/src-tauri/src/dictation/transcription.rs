use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::{CLEANUP_MODEL, DictationConfiguration, delivery, history, panel, publish_status};
use crate::audio::finalize_microphone_track;
use crate::error::{AppError, AppResult};
use crate::local_models::installed_path;
use crate::state::AppState;

use opentranscribe_domain::{DictationHistoryEntry, DictationPhase, DictationProvider};
use opentranscribe_transcriber_protocol::{Command, Event};
use tauri::Manager;

#[derive(Clone)]
pub(super) struct Transcript {
    pub(super) text: String,
    pub(super) model_id: String,
    pub(super) usage: Option<serde_json::Value>,
    pub(super) cost: Option<f64>,
}

pub(super) async fn prepare(
    app: &tauri::AppHandle,
    config: &DictationConfiguration,
    canceled: &AtomicBool,
) -> AppResult<()> {
    let state = app.state::<AppState>();
    let mut workers = state.dictation_workers.lock().await;
    if config.provider == DictationProvider::Local {
        let model = config
            .local_model_id
            .as_deref()
            .ok_or(AppError::RecordingInactive)?;
        workers
            .speech
            .prepare(
                app,
                "local-transcriber",
                model,
                &installed_path(app, model)?,
                canceled,
            )
            .await?;
    }
    if config.provider != DictationProvider::Local {
        workers.speech.unload();
    }
    if !config.cleanup {
        workers.cleanup.unload();
    }
    if config.cleanup {
        workers
            .cleanup
            .prepare(
                app,
                "text-normalizer",
                CLEANUP_MODEL,
                &installed_path(app, CLEANUP_MODEL)?,
                canceled,
            )
            .await?;
    }
    Ok(())
}

pub(super) async fn run(
    app: tauri::AppHandle,
    id: String,
    directory: PathBuf,
    config: DictationConfiguration,
    canceled: Arc<AtomicBool>,
    previous: Option<Transcript>,
) {
    let result = process(&app, &id, &directory, &config, canceled.clone(), previous).await;
    let state = app.state::<AppState>();
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    if canceled.load(Ordering::SeqCst) || run.status.id.as_deref() != Some(&id) {
        drop(run);
        history::remove_recovery_directory(&directory, "canceled");
        return;
    }
    let auto_pasted = match result {
        Ok((text, auto_pasted)) => {
            run.status.phase = DictationPhase::Completed;
            run.status.text = Some(text);
            run.status.auto_pasted = auto_pasted;
            run.status.can_retry = false;
            run.status.progress_percent = None;
            run.recovery_directory = None;
            run.configuration = None;
            run.transcript = None;
            history::remove_recovery_directory(&directory, "completed");
            auto_pasted
        }
        Err(error) => {
            run.status.phase = DictationPhase::Failed;
            run.status.can_retry = true;
            run.status.error_message = Some(error.to_string());
            false
        }
    };
    drop(run);
    publish_status(&state);
    if auto_pasted {
        panel::hide(&app);
    } else if let Err(error) = panel::show(&app, false) {
        log::warn!("could not show dictation result: {error}");
    }
}

async fn process(
    app: &tauri::AppHandle,
    id: &str,
    directory: &std::path::Path,
    config: &DictationConfiguration,
    canceled: Arc<AtomicBool>,
    previous: Option<Transcript>,
) -> AppResult<(String, bool)> {
    let transcript = match previous {
        Some(transcript) => transcript,
        None => {
            let directory = directory.to_owned();
            let path = tauri::async_runtime::spawn_blocking(move || {
                let path = directory.join("dictation.wav");
                finalize_microphone_track(&directory, &path)?;
                Ok::<_, AppError>(path)
            })
            .await
            .map_err(|error| AppError::Audio(error.to_string()))??;
            check_canceled(&canceled)?;
            super::providers::transcribe(app, id, path, config, canceled.clone()).await?
        }
    };
    check_canceled(&canceled)?;
    {
        let state = app.state::<AppState>();
        let mut run = state.dictation.lock().expect("dictation lock poisoned");
        if run.status.id.as_deref() != Some(id) {
            return Err(AppError::JobCanceled);
        }
        run.transcript = Some(transcript.clone());
        run.status.text = Some(transcript.text.clone());
        run.status.approximate_cost_usd = transcript.cost;
    }
    if transcript.text.trim().is_empty() {
        return Ok((String::new(), false));
    }
    // Save the original before optional cleanup, so a model failure cannot lose recognized words.
    save_transcript(app, id, config, &transcript, transcript.text.clone(), false)?;
    let text = if config.cleanup && !transcript.text.is_empty() {
        update_progress(app, id, DictationPhase::Cleaning, None);
        let state = app.state::<AppState>();
        let mut workers = state.dictation_workers.lock().await;
        workers
            .cleanup
            .prepare(
                app,
                "text-normalizer",
                CLEANUP_MODEL,
                &installed_path(app, CLEANUP_MODEL)?,
                &canceled,
            )
            .await?;
        let events = workers
            .cleanup
            .request(
                Command::NormalizeText {
                    job_id: id.to_owned(),
                    text: transcript.text.clone(),
                },
                &canceled,
                |_| {},
            )
            .await?;
        events
            .into_iter()
            .find_map(|event| match event {
                Event::NormalizedText { text, .. } => Some(text),
                _ => None,
            })
            .ok_or_else(|| AppError::Model("S1-mini returned no completion response.".to_owned()))?
    } else {
        transcript.text.clone()
    };
    check_canceled(&canceled)?;
    save_transcript(app, id, config, &transcript, text.clone(), config.cleanup)?;
    if text.is_empty() {
        return Ok((text, false));
    }
    let pasted = delivery::deliver(
        app,
        text.clone(),
        config.auto_paste,
        config.target,
        canceled,
    )
    .await?;
    Ok((text, pasted))
}

fn save_transcript(
    app: &tauri::AppHandle,
    id: &str,
    config: &DictationConfiguration,
    transcript: &Transcript,
    text: String,
    cleaned: bool,
) -> AppResult<()> {
    history::save_history(
        app,
        DictationHistoryEntry {
            id: id.to_owned(),
            text,
            raw_text: cleaned.then(|| transcript.text.clone()),
            cleanup_model_id: cleaned.then(|| CLEANUP_MODEL.to_owned()),
            created_at: opentranscribe_domain::now(),
            provider: config.provider.clone(),
            model_id: transcript.model_id.clone(),
            usage: transcript.usage.clone(),
            approximate_cost_usd: transcript.cost,
        },
    )
}

pub(super) fn check_canceled(canceled: &AtomicBool) -> AppResult<()> {
    if canceled.load(Ordering::SeqCst) {
        Err(AppError::JobCanceled)
    } else {
        Ok(())
    }
}

pub(super) fn update_progress(
    app: &tauri::AppHandle,
    id: &str,
    phase: DictationPhase,
    progress: Option<u8>,
) {
    let state = app.state::<AppState>();
    let mut run = state.dictation.lock().expect("dictation lock poisoned");
    if run.status.id.as_deref() != Some(id) || run.canceled.load(Ordering::SeqCst) {
        return;
    }
    run.status.phase = phase;
    run.status.progress_percent = progress;
    drop(run);
    publish_status(&state);
}
