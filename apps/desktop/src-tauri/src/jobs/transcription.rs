use opentranscribe_domain::{
    AppEvent, JobProgress, JobStage, ProgressUnit, TranscriptionPreviewUpdate,
};
use tauri::Manager;

use super::{complete_job, is_job_canceled, prepare_job, report_progress};
use crate::error::AppResult;
use crate::events::send_event;
use crate::local_models::LocalTranscriptionService;
use crate::state::AppState;
use crate::transcription::OpenAiTranscriptionService;

pub(super) fn run_openai_job(
    app: &tauri::AppHandle,
    job_id: &str,
    model_id: String,
    live_stream_count: u8,
) -> AppResult<()> {
    let input = prepare_job(app, job_id)?;
    let session_id = input.session.id.clone();
    let api_key = app.state::<AppState>().openai_credentials.read()?;
    let bundle = OpenAiTranscriptionService::run(
        input,
        &api_key,
        model_id,
        live_stream_count,
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
        |text| {
            send_event(
                &app.state::<AppState>(),
                AppEvent::TranscriptionPreviewChanged(TranscriptionPreviewUpdate {
                    session_id: session_id.clone(),
                    job_id: job_id.to_owned(),
                    text: text.to_owned(),
                }),
            );
        },
        || is_job_canceled(app, job_id),
    )?;
    complete_job(app, job_id, |repository| {
        repository
            .save_transcript(&bundle.transcript, &bundle.run, &bundle.provider_response)
            .map(|_| ())
    })
}

pub(super) async fn run_local_job(
    app: &tauri::AppHandle,
    job_id: &str,
    model_id: String,
    diarization_model_id: Option<String>,
) -> AppResult<()> {
    let input = prepare_job(app, job_id)?;
    let bundle = LocalTranscriptionService::run(
        app,
        input,
        model_id,
        diarization_model_id,
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
    complete_job(app, job_id, |repository| {
        repository
            .save_transcript(&bundle.transcript, &bundle.run, &bundle.provider_response)
            .map(|_| ())
    })
}
