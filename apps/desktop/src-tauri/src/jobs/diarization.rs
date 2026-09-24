use opentranscribe_domain::{AppEvent, Job, JobProgress, JobStage, ProgressUnit, Transcript};
use tauri::Manager;

use crate::error::{AppError, AppResult};
use crate::events::send_event;
use crate::local_models::diarizer;
use crate::state::{AppState, with_repository};
use crate::storage::JobRequest;

pub(crate) struct DiarizationRequest {
    pub session_id: String,
    pub model_id: String,
    pub transcript_id: String,
    pub expected_revision: u64,
}

pub(crate) fn enqueue(
    request: DiarizationRequest,
    app: tauri::AppHandle,
    state: &AppState,
) -> AppResult<Job> {
    diarizer::model_path(&app, &request.model_id)?;
    let record = with_repository(state, |repository| {
        let input = repository.transcription_input(&request.session_id)?;
        if input.audio_files.is_empty() {
            return Err(AppError::Application(
                "Keep the session audio to identify speakers.".to_owned(),
            ));
        }
        let transcript = repository.diarization_input(
            &request.session_id,
            &request.transcript_id,
            request.expected_revision,
        )?;
        repository.create_job(
            request.session_id,
            JobRequest::DiarizeLocal {
                model_id: request.model_id,
                transcript: Box::new(transcript),
            },
        )
    })?;
    send_event(state, AppEvent::JobStateChanged(record.job.clone()));
    super::spawn_job(app, record.clone());
    Ok(record.job)
}

pub(super) async fn run(
    app: &tauri::AppHandle,
    job_id: &str,
    model_id: &str,
    transcript: &Transcript,
) -> AppResult<()> {
    let input = super::prepare_job(app, job_id)?;
    // A queued or retried job must still refer to the transcript it was requested against.
    with_repository(&app.state::<AppState>(), |repository| {
        let current =
            repository.diarization_input(&input.session.id, &transcript.id, transcript.revision)?;
        if current != *transcript {
            return Err(AppError::RevisionConflict);
        }
        Ok(())
    })?;
    let audio_path = input
        .audio_files
        .first()
        .ok_or_else(|| AppError::Application("The session audio is missing.".to_owned()))?;
    super::report_progress(
        app,
        job_id,
        JobProgress {
            stage: JobStage::Diarizing,
            completed_units: 0,
            total_units: None,
            unit: ProgressUnit::AudioMs,
            message: "Identifying speakers locally".to_owned(),
        },
    );
    let turns = diarizer::run(app, audio_path, model_id, || {
        super::is_job_canceled(app, job_id)
    })
    .await?;
    super::complete_job(app, job_id, |repository| {
        repository
            .save_diarization(
                transcript,
                model_id,
                diarizer::model_sha256(model_id)?,
                &turns,
            )
            .map(|_| ())
    })
}
