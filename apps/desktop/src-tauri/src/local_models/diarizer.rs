use std::path::{Path, PathBuf};

use opentranscribe_transcriber_protocol::{Command, FileDiarization, SpeakerTurn};

use super::{catalog, installed_path, sidecar::run_sidecar};
use crate::error::{AppError, AppResult};

pub(crate) fn model_path(app: &tauri::AppHandle, model_id: &str) -> AppResult<PathBuf> {
    if model_id != catalog::NEMOTRON_MODEL_ID {
        return Err(AppError::Model(
            "Select a speaker recognition model for diarization.".to_owned(),
        ));
    }
    installed_path(app, model_id)
}

pub(crate) fn model_sha256(model_id: &str) -> AppResult<&'static str> {
    catalog::find(model_id)
        .filter(|model| model.preset == "diarization")
        .map(|model| model.sha256)
        .ok_or_else(|| AppError::Model("unknown diarization model".to_owned()))
}

pub(crate) async fn run(
    app: &tauri::AppHandle,
    audio_path: &Path,
    model_id: &str,
    should_cancel: impl Fn() -> bool,
) -> AppResult<Vec<SpeakerTurn>> {
    let path = model_path(app, model_id)?;
    let job_id = opentranscribe_domain::new_id();
    let output = run_sidecar(
        app,
        None,
        &job_id,
        Command::DiarizeFile(FileDiarization {
            job_id: job_id.clone(),
            path: audio_path.to_string_lossy().into_owned(),
            model_path: path.to_string_lossy().into_owned(),
        }),
        |_, _| {},
        should_cancel,
    )
    .await?;
    Ok(output.speaker_turns)
}
