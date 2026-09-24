pub(crate) mod diarization;
mod finalization;
mod transcription;

use opentranscribe_domain::{
    AppEvent, ArtifactKind, Job, JobProgress, JobStage, JobState, ProgressUnit, RecordingMode,
};
use tauri::Manager;

use crate::error::{AppError, AppResult};
use crate::events::send_event;
use crate::local_models::{installed_path, preferred_installed_model_id};
use crate::state::{AppState, with_repository};
use crate::storage::{JobRecord, JobRequest};

pub(crate) enum TranscriptionProvider {
    Local,
    OpenAi,
}

pub(crate) struct TranscriptionRequest {
    pub(crate) session_id: String,
    pub(crate) provider: TranscriptionProvider,
    pub(crate) model_id: String,
    pub(crate) diarization_model_id: Option<String>,
}

pub(crate) fn enqueue_recording_finalization(
    app: &tauri::AppHandle,
    session_id: &str,
    mode: RecordingMode,
    openai_model_id: String,
) -> AppResult<Job> {
    let state = app.state::<AppState>();
    let record = with_repository(&state, |repository| {
        repository.create_job(
            session_id.to_owned(),
            JobRequest::FinalizeRecording {
                mode,
                openai_model_id,
            },
        )
    })?;
    send_event(&state, AppEvent::JobStateChanged(record.job.clone()));
    spawn_job(app.clone(), record.clone());
    Ok(record.job)
}

pub(crate) fn enqueue_post_recording_transcription(
    app: &tauri::AppHandle,
    session_id: &str,
    mode: RecordingMode,
    openai_model_id: String,
) -> AppResult<Option<Job>> {
    let request = match mode {
        RecordingMode::RecordOnly => return Ok(None),
        RecordingMode::LocalAfterRecording => TranscriptionRequest {
            session_id: session_id.to_owned(),
            provider: TranscriptionProvider::Local,
            model_id: preferred_installed_model_id(app)?,
            diarization_model_id: None,
        },
        RecordingMode::OpenAiLive => TranscriptionRequest {
            session_id: session_id.to_owned(),
            provider: TranscriptionProvider::OpenAi,
            model_id: openai_model_id,
            diarization_model_id: None,
        },
    };

    enqueue_transcription(
        request,
        mode == RecordingMode::OpenAiLive,
        app.clone(),
        &app.state(),
    )
    .map(Some)
}

pub(crate) fn enqueue_transcription(
    request: TranscriptionRequest,
    includes_live_transcription: bool,
    app: tauri::AppHandle,
    state: &AppState,
) -> AppResult<Job> {
    let input = with_repository(state, |repository| {
        repository.transcription_input(&request.session_id)
    })?;

    if input.audio_files.is_empty() {
        return Err(AppError::Application(
            "record or import audio before requesting a transcription".to_owned(),
        ));
    }

    let live_stream_count = if includes_live_transcription {
        1 + u8::from(
            input
                .session
                .artifacts
                .iter()
                .any(|artifact| artifact.kind == ArtifactKind::System),
        )
    } else {
        0
    };
    let estimated_cost_usd = match request.provider {
        TranscriptionProvider::Local => None,
        TranscriptionProvider::OpenAi => crate::transcription::estimate_openai_cost(
            &request.model_id,
            input.session.duration_ms,
            live_stream_count,
        ),
    };
    let job_request = match request.provider {
        TranscriptionProvider::Local => {
            installed_path(&app, &request.model_id)?;
            if let Some(id) = &request.diarization_model_id {
                crate::local_models::diarizer::model_path(&app, id)?;
            }
            JobRequest::TranscribeLocal {
                model_id: request.model_id,
                diarization_model_id: request.diarization_model_id,
            }
        }
        TranscriptionProvider::OpenAi => {
            if request.diarization_model_id.is_some() {
                return Err(AppError::Application(
                    "Use speaker recognition after cloud transcription completes.".to_owned(),
                ));
            }
            state.openai_credentials.read()?;
            JobRequest::TranscribeOpenAi {
                model_id: request.model_id,
                live_stream_count,
            }
        }
    };
    let record = with_repository(state, |repository| {
        let record = repository.create_job(request.session_id, job_request)?;
        repository.update_job(&record.job.id, |job| {
            job.estimated_cost_usd = estimated_cost_usd;
        })
    })?;
    send_event(state, AppEvent::JobStateChanged(record.job.clone()));
    spawn_job(app, record.clone());
    Ok(record.job)
}

pub(crate) fn retry_job(job_id: &str, app: tauri::AppHandle, state: &AppState) -> AppResult<Job> {
    state
        .canceled_jobs
        .lock()
        .expect("app state lock poisoned")
        .remove(job_id);
    let record = with_repository(state, |repository| {
        let current = repository.job_record(job_id)?;

        if current.job.state != JobState::Failed {
            return Err(AppError::Application(
                "only failed jobs can be retried".to_owned(),
            ));
        }

        repository.update_job(job_id, |job| {
            job.state = JobState::Queued;
            job.progress = None;
            job.error_message = None;
            job.attempt += 1;
        })
    })?;
    send_event(state, AppEvent::JobStateChanged(record.job.clone()));
    spawn_job(app, record.clone());
    Ok(record.job)
}

pub(crate) fn cancel_job(job_id: &str, state: &AppState) -> AppResult<Job> {
    let mut canceled = state.canceled_jobs.lock().expect("app state lock poisoned");
    let current = with_repository(state, |repository| repository.job_record(job_id))?;

    if current.job.kind == opentranscribe_domain::JobKind::FinalizeRecording {
        return Err(AppError::Application(
            "recording finalization cannot be canceled".to_owned(),
        ));
    }

    if !matches!(
        current.job.state,
        JobState::Queued | JobState::Preparing | JobState::Running | JobState::Failed
    ) {
        return Err(AppError::Application(
            "only active or failed jobs can be canceled".to_owned(),
        ));
    }

    canceled.insert(job_id.to_owned());
    let record = with_repository(state, |repository| {
        repository.update_job(job_id, |job| {
            job.state = JobState::Canceled;
            job.progress = None;
            job.error_message = None;
        })
    })?;
    drop(canceled);
    send_event(state, AppEvent::JobStateChanged(record.job.clone()));
    Ok(record.job)
}

fn spawn_job(app: tauri::AppHandle, record: JobRecord) {
    match record.request.clone() {
        JobRequest::FinalizeRecording {
            mode,
            openai_model_id,
        } => {
            tauri::async_runtime::spawn_blocking(move || {
                if let Err(error) = finalization::run_recording_finalization_job(
                    &app,
                    &record.job.id,
                    mode,
                    openai_model_id,
                ) {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
        JobRequest::TranscribeOpenAi {
            model_id,
            live_stream_count,
        } => {
            tauri::async_runtime::spawn_blocking(move || {
                if let Err(error) =
                    transcription::run_openai_job(&app, &record.job.id, model_id, live_stream_count)
                {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
        JobRequest::TranscribeLocal {
            model_id,
            diarization_model_id,
        } => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) = transcription::run_local_job(
                    &app,
                    &record.job.id,
                    model_id,
                    diarization_model_id,
                )
                .await
                {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
        JobRequest::DiarizeLocal {
            model_id,
            transcript,
        } => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) =
                    diarization::run(&app, &record.job.id, &model_id, &transcript).await
                {
                    fail_job(&app, &record.job.id, error);
                }
            });
        }
    }
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
    save: impl FnOnce(&crate::storage::LibraryRepository) -> AppResult<()>,
) -> AppResult<()> {
    let state = app.state::<AppState>();
    // Cancellation and promotion share one lock; canceled work cannot change a transcript.
    let canceled = state.canceled_jobs.lock().expect("app state lock poisoned");
    if canceled.contains(job_id) {
        return Err(AppError::JobCanceled);
    }
    let record = with_repository(&state, |repository| {
        save(repository)?;
        repository.update_job(job_id, |job| {
            job.state = JobState::Completed;
            job.progress = Some(JobProgress {
                stage: JobStage::Finalizing,
                completed_units: 1,
                total_units: Some(1),
                unit: ProgressUnit::Items,
                message: "Transcript updated".to_owned(),
            });
        })
    })?;
    drop(canceled);
    send_event(&state, AppEvent::JobStateChanged(record.job));
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
    let canceled = state.canceled_jobs.lock().expect("app state lock poisoned");
    if canceled.contains(job_id) {
        return Err(AppError::JobCanceled);
    }
    let record = with_repository(&state, |repository| repository.update_job(job_id, update))?;
    drop(canceled);
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
