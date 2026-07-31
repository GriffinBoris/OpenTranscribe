use opentranscribe_domain::{
    AppEvent, Job, JobProgress, JobStage, JobState, ProgressUnit, RecordingMode,
};
use serde::Deserialize;
use tauri::Manager;

use crate::commands::{AppState, send_event, with_repository};
use crate::credentials::OpenAiCredentials;
use crate::error::{AppError, AppResult};
use crate::local_models::{
    LocalTranscriptionService, installed_path, preferred_installed_model_id,
};
use crate::storage::{JobRecord, JobRequest};
use crate::transcription::OpenAiTranscriptionService;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionProvider {
    Local,
    OpenAi,
}

#[derive(Deserialize)]
pub struct EnqueueTranscriptionRequest {
    pub session_id: String,
    pub provider: TranscriptionProvider,
    pub model_id: String,
}

pub(crate) fn enqueue_post_recording_transcription(
    app: &tauri::AppHandle,
    session_id: &str,
    mode: RecordingMode,
    openai_model_id: String,
) -> AppResult<Option<Job>> {
    let request = match mode {
        RecordingMode::RecordOnly => return Ok(None),
        RecordingMode::LocalLive => EnqueueTranscriptionRequest {
            session_id: session_id.to_owned(),
            provider: TranscriptionProvider::Local,
            model_id: preferred_installed_model_id(app)?,
        },
        RecordingMode::OpenAiLive => EnqueueTranscriptionRequest {
            session_id: session_id.to_owned(),
            provider: TranscriptionProvider::OpenAi,
            model_id: openai_model_id,
        },
    };

    enqueue_transcription_job(request, app.clone(), app.state()).map(Some)
}

#[tauri::command]
pub fn enqueue_transcription(
    request: EnqueueTranscriptionRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    enqueue_transcription_job(request, app, state)
}

fn enqueue_transcription_job(
    request: EnqueueTranscriptionRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    let input = with_repository(&state, |repository| {
        repository.transcription_input(&request.session_id)
    })?;

    if input.audio_files.is_empty() {
        return Err(AppError::Application(
            "record or import audio before requesting a transcription".to_owned(),
        ));
    }

    let job_request = match request.provider {
        TranscriptionProvider::Local => {
            installed_path(&app, &request.model_id)?;
            JobRequest::TranscribeLocal {
                model_id: request.model_id,
            }
        }
        TranscriptionProvider::OpenAi => {
            OpenAiCredentials::read()?;
            JobRequest::TranscribeOpenAi {
                model_id: request.model_id,
            }
        }
    };
    let record = with_repository(&state, |repository| {
        repository.create_job(request.session_id, job_request)
    })?;
    send_event(&state, AppEvent::JobStateChanged(record.job.clone()));
    spawn_job(app, record.clone());
    Ok(record.job)
}

#[tauri::command]
pub fn retry_job(
    job_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Job> {
    state
        .canceled_jobs
        .lock()
        .expect("app state lock poisoned")
        .remove(&job_id);
    let record = with_repository(&state, |repository| {
        let current = repository.job_record(&job_id)?;

        if current.job.state != JobState::Failed {
            return Err(AppError::Application(
                "only failed jobs can be retried".to_owned(),
            ));
        }

        repository.update_job(&job_id, |job| {
            job.state = JobState::Queued;
            job.progress = None;
            job.error_message = None;
            job.attempt += 1;
        })
    })?;
    send_event(&state, AppEvent::JobStateChanged(record.job.clone()));
    spawn_job(app, record.clone());
    Ok(record.job)
}

#[tauri::command]
pub fn cancel_job(job_id: String, state: tauri::State<'_, AppState>) -> AppResult<Job> {
    let current = with_repository(&state, |repository| repository.job_record(&job_id))?;

    if !matches!(
        current.job.state,
        JobState::Queued | JobState::Preparing | JobState::Running
    ) {
        return Err(AppError::Application(
            "only an active job can be canceled".to_owned(),
        ));
    }

    state
        .canceled_jobs
        .lock()
        .expect("app state lock poisoned")
        .insert(job_id.clone());
    let record = with_repository(&state, |repository| {
        repository.update_job(&job_id, |job| {
            job.state = JobState::Canceled;
            job.progress = None;
            job.error_message = None;
        })
    })?;
    send_event(&state, AppEvent::JobStateChanged(record.job.clone()));
    Ok(record.job)
}

fn spawn_job(app: tauri::AppHandle, record: JobRecord) {
    match record.request.clone() {
        JobRequest::TranscribeOpenAi { model_id } => {
            tauri::async_runtime::spawn_blocking(move || {
                if let Err(error) = run_openai_job(&app, &record.job.id, model_id) {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
        JobRequest::TranscribeLocal { model_id } => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) = run_local_job(&app, &record.job.id, model_id).await {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
    }
}

fn run_openai_job(app: &tauri::AppHandle, job_id: &str, model_id: String) -> AppResult<()> {
    let input = prepare_job(app, job_id)?;
    let api_key = OpenAiCredentials::read()?;
    let bundle = OpenAiTranscriptionService::run(
        input,
        &api_key,
        model_id,
        |completed, total| {
            report_progress(
                app,
                job_id,
                JobProgress {
                    stage: JobStage::Transcribing,
                    completed_units: completed as u64,
                    total_units: Some(total as u64),
                    unit: ProgressUnit::Items,
                    message: format!("Transcribed {completed} of {total} audio chunks"),
                },
            );
        },
        || is_job_canceled(app, job_id),
    )?;
    complete_job(app, job_id, bundle)
}

async fn run_local_job(app: &tauri::AppHandle, job_id: &str, model_id: String) -> AppResult<()> {
    let input = prepare_job(app, job_id)?;
    let bundle = LocalTranscriptionService::run(
        app,
        input,
        model_id,
        |completed, total| {
            let percent = completed.saturating_mul(100) / total.max(1);
            report_progress(
                app,
                job_id,
                JobProgress {
                    stage: JobStage::Transcribing,
                    completed_units: completed,
                    total_units: Some(total),
                    unit: ProgressUnit::AudioMs,
                    message: format!("Transcribing locally · {percent}%"),
                },
            );
        },
        || is_job_canceled(app, job_id),
    )
    .await?;
    complete_job(app, job_id, bundle)
}

fn prepare_job(
    app: &tauri::AppHandle,
    job_id: &str,
) -> AppResult<crate::storage::TranscriptionInput> {
    check_job_canceled(app, job_id)?;
    update_job(app, job_id, |job| {
        job.state = JobState::Preparing;
        job.error_message = None;
        job.progress = Some(JobProgress {
            stage: JobStage::Preparing,
            completed_units: 0,
            total_units: None,
            unit: ProgressUnit::Items,
            message: "Preparing transcription".to_owned(),
        });
    })?;
    let state = app.state::<AppState>();
    let record = with_repository(&state, |repository| repository.job_record(job_id))?;
    let session_id = record
        .job
        .session_id
        .as_deref()
        .expect("transcription jobs have a session");
    let input = with_repository(&state, |repository| {
        repository.transcription_input(session_id)
    })?;
    check_job_canceled(app, job_id)?;
    update_job(app, job_id, |job| job.state = JobState::Running)?;
    Ok(input)
}

fn complete_job(
    app: &tauri::AppHandle,
    job_id: &str,
    bundle: crate::transcription::TranscriptionBundle,
) -> AppResult<()> {
    check_job_canceled(app, job_id)?;
    let state = app.state::<AppState>();
    with_repository(&state, |repository| {
        repository.save_transcript(&bundle.transcript, &bundle.run, &bundle.provider_response)
    })?;
    update_job(app, job_id, |job| {
        job.state = JobState::Completed;
        job.progress = Some(JobProgress {
            stage: JobStage::Finalizing,
            completed_units: 1,
            total_units: Some(1),
            unit: ProgressUnit::Items,
            message: "Transcription complete".to_owned(),
        });
    })?;
    send_event(&state, AppEvent::LibraryChanged);
    Ok(())
}

fn report_progress(app: &tauri::AppHandle, job_id: &str, progress: JobProgress) {
    if is_job_canceled(app, job_id) {
        return;
    }

    let progress_event = progress.clone();

    if update_job(app, job_id, |job| {
        job.state = JobState::Running;
        job.progress = Some(progress);
    })
    .is_ok()
    {
        let state = app.state::<AppState>();
        send_event(
            &state,
            AppEvent::JobProgress {
                job_id: job_id.to_owned(),
                progress: progress_event,
            },
        );
    }
}

fn update_job(
    app: &tauri::AppHandle,
    job_id: &str,
    update: impl FnOnce(&mut Job),
) -> AppResult<Job> {
    let state = app.state::<AppState>();
    let record = with_repository(&state, |repository| repository.update_job(job_id, update))?;
    send_event(&state, AppEvent::JobStateChanged(record.job.clone()));
    Ok(record.job)
}

fn fail_job(app: &tauri::AppHandle, job_id: &str, error: AppError) {
    if is_job_canceled(app, job_id) || matches!(error, AppError::JobCanceled) {
        return;
    }

    if let Err(update_error) = update_job(app, job_id, |job| {
        job.state = JobState::Failed;
        job.error_message = Some(error.to_string());
    }) {
        log::error!("failed to persist transcription job failure: {update_error}");
    }
}

fn check_job_canceled(app: &tauri::AppHandle, job_id: &str) -> AppResult<()> {
    if is_job_canceled(app, job_id) {
        return Err(AppError::JobCanceled);
    }

    Ok(())
}

fn is_job_canceled(app: &tauri::AppHandle, job_id: &str) -> bool {
    app.state::<AppState>()
        .canceled_jobs
        .lock()
        .expect("app state lock poisoned")
        .contains(job_id)
}
