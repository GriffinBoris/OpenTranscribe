use super::DictationConfiguration;
use super::transcription::{Transcript, check_canceled, update_progress};
use crate::error::{AppError, AppResult};
use crate::local_models::installed_path;
use crate::state::AppState;
use crate::transcription::{
    OpenAiFileTranscriber, OpenAiTranscriptionRequest, estimate_openai_cost, openai_usage,
};
use opentranscribe_domain::{DictationPhase, DictationProvider};
use opentranscribe_transcriber_protocol::{Command, Event, FileTranscription};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::Manager;
pub(super) async fn transcribe(
    app: &tauri::AppHandle,
    id: &str,
    path: PathBuf,
    config: &DictationConfiguration,
    canceled: Arc<AtomicBool>,
) -> AppResult<Transcript> {
    if config.provider == DictationProvider::Local {
        let model = config
            .local_model_id
            .as_deref()
            .ok_or(AppError::RecordingInactive)?;
        let state = app.state::<AppState>();
        let mut workers = state.dictation_workers.lock().await;
        workers
            .speech
            .prepare(
                app,
                "local-transcriber",
                model,
                &installed_path(app, model)?,
                &canceled,
            )
            .await?;
        let events = workers
            .speech
            .request(
                Command::TranscribeFile(FileTranscription {
                    job_id: id.to_owned(),
                    path: path.to_string_lossy().into_owned(),
                    language_hint: None,
                    prompt: None,
                }),
                &canceled,
                |event| {
                    if let Event::JobProgress {
                        completed_ms,
                        total_ms,
                        ..
                    } = event
                        && *total_ms > 0
                    {
                        update_progress(
                            app,
                            id,
                            DictationPhase::Transcribing,
                            Some((completed_ms.saturating_mul(100) / total_ms).min(100) as u8),
                        );
                    }
                },
            )
            .await?;
        let text = events
            .into_iter()
            .filter_map(|event| match event {
                Event::Segment { text, .. } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        return Ok(Transcript {
            text,
            model_id: model.to_owned(),
            usage: None,
            cost: None,
        });
    }
    let key = app.state::<AppState>().openai_credentials.read()?;
    let model_id = config.openai_model.model_id().to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let reader =
            hound::WavReader::open(&path).map_err(|error| AppError::Audio(error.to_string()))?;
        let duration = u64::from(reader.duration()) * 1_000 / u64::from(reader.spec().sample_rate);
        let transcripts = OpenAiFileTranscriber::new().transcribe(
            OpenAiTranscriptionRequest {
                api_key: &key,
                model: &model_id,
                files: &[path],
                language_hint: None,
            },
            |_, _| {},
            |_| {},
            || canceled.load(Ordering::SeqCst),
        )?;
        check_canceled(&canceled)?;
        let text = transcripts
            .iter()
            .map(|part| part.text.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let usage = openai_usage(
            &model_id,
            duration,
            0,
            transcripts
                .iter()
                .filter_map(|part| part.usage.clone())
                .collect(),
        );
        let cost = estimate_openai_cost(&model_id, duration, 0);
        Ok(Transcript {
            text,
            model_id,
            usage: Some(usage),
            cost,
        })
    })
    .await
    .map_err(|error| AppError::Provider(error.to_string()))?
}
