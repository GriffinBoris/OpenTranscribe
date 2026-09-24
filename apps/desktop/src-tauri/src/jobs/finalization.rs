use opentranscribe_domain::{
    AppEvent, JobProgress, JobStage, JobState, ProgressUnit, RecordingMode,
};
use tauri::Manager;

use super::{enqueue_post_recording_transcription, update_job};
use crate::error::AppResult;
use crate::events::send_event;
use crate::state::{AppState, with_repository};

pub(super) fn run_recording_finalization_job(
    app: &tauri::AppHandle,
    job_id: &str,
    mode: RecordingMode,
    openai_model_id: String,
) -> AppResult<()> {
    update_job(app, job_id, |job| {
        job.state = JobState::Running;
        job.progress = Some(JobProgress {
            stage: JobStage::Finalizing,
            completed_units: 0,
            total_units: Some(1),
            unit: ProgressUnit::Items,
            message: "Finalizing recording".to_owned(),
        });
    })?;
    let state = app.state::<AppState>();
    let record = with_repository(&state, |repository| repository.job_record(job_id))?;
    let session_id = record
        .job
        .session_id
        .as_deref()
        .expect("recording finalization jobs have a session");
    let session = with_repository(&state, |repository| repository.finish_recording(session_id))
        .map_err(|error| {
            crate::recording::handle_finalization_failure(app, &state, session_id, error)
        })?;

    update_job(app, job_id, |job| {
        job.state = JobState::Completed;
        job.progress = Some(JobProgress {
            stage: JobStage::Finalizing,
            completed_units: 1,
            total_units: Some(1),
            unit: ProgressUnit::Items,
            message: "Recording ready".to_owned(),
        });
    })?;
    send_event(&state, AppEvent::LibraryChanged);

    if let Err(error) =
        enqueue_post_recording_transcription(app, &session.id, mode, openai_model_id)
    {
        let message =
            format!("Recording saved, but automatic transcription could not start: {error}");
        log::warn!("{message}");
        send_event(&state, AppEvent::AttentionRequired(message));
    }

    Ok(())
}
