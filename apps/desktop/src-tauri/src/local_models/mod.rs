mod catalog;
mod manager;
mod transcriber;

use std::path::PathBuf;

use opentranscribe_domain::{JobProgress, JobStage, ProgressUnit};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug, Serialize)]
pub struct LocalModel {
    pub id: String,
    pub preset: String,
    pub label: String,
    pub description: String,
    pub byte_count: u64,
    pub installed: bool,
}

#[derive(Deserialize)]
pub struct MoveLocalModelsRequest {
    pub path: String,
}

#[tauri::command]
pub fn local_model_statuses(app: tauri::AppHandle) -> AppResult<Vec<LocalModel>> {
    manager::statuses(&app)
}

#[tauri::command]
pub fn local_model_storage_path(app: tauri::AppHandle) -> AppResult<String> {
    Ok(manager::models_root(&app)?.display().to_string())
}

#[tauri::command]
pub fn move_local_models(
    request: MoveLocalModelsRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
) -> AppResult<String> {
    let target = PathBuf::from(&request.path);

    if !target.is_absolute() || target.parent().is_none() {
        return Err(AppError::Model(
            "choose a folder for local models instead of a filesystem root".to_owned(),
        ));
    }

    if state.recorder.status().is_some() {
        return Err(AppError::Application(
            "stop the current recording before moving local models".to_owned(),
        ));
    }

    let has_running_jobs = state
        .repository
        .lock()
        .expect("app state lock poisoned")
        .as_ref()
        .map(crate::storage::LibraryRepository::has_running_jobs)
        .transpose()?
        .unwrap_or(false);

    if has_running_jobs {
        return Err(AppError::Application(
            "wait for active processing to finish before moving local models".to_owned(),
        ));
    }

    if !state
        .active_model_downloads
        .lock()
        .expect("app state lock poisoned")
        .is_empty()
    {
        return Err(AppError::Application(
            "wait for local model downloads to finish before moving them".to_owned(),
        ));
    }

    let source = manager::move_models(&app, &target)?;
    if let Err(error) = crate::commands::application::update_settings(&app, &state, |settings| {
        settings.local_models_directory = Some(target.display().to_string());
    }) {
        if let Some(source) = source
            && let Err(rollback_error) = manager::move_models_between(&target, &source)
        {
            return Err(AppError::Model(format!(
                "local models moved to {} but saving the new location failed: {error}; unable to restore them: {rollback_error}",
                target.display(),
            )));
        }

        return Err(error);
    }
    Ok(target.display().to_string())
}

#[tauri::command]
pub async fn download_local_model(
    app: tauri::AppHandle,
    model_id: String,
    progress_channel: Channel<JobProgress>,
    state: tauri::State<'_, crate::state::AppState>,
) -> AppResult<LocalModel> {
    if !state
        .active_model_downloads
        .lock()
        .expect("app state lock poisoned")
        .insert(model_id.clone())
    {
        return Err(AppError::Model(
            "that local model is already downloading".to_owned(),
        ));
    }

    let download_model_id = model_id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut progress_connected = true;

        manager::download(&app, &download_model_id, |completed, total| {
            if !progress_connected {
                return;
            }

            let percent = completed.saturating_mul(100) / total;
            let progress = JobProgress {
                stage: JobStage::Downloading,
                completed_units: completed,
                total_units: Some(total),
                unit: ProgressUnit::Bytes,
                message: format!("Downloading model · {percent}%"),
            };

            if let Err(error) = progress_channel.send(progress) {
                log::warn!("model progress subscriber disconnected: {error}");
                progress_connected = false;
            }
        })
    })
    .await
    .map_err(|error| AppError::Model(error.to_string()))
    .and_then(|result| result);

    state
        .active_model_downloads
        .lock()
        .expect("app state lock poisoned")
        .remove(&model_id);
    result
}

#[tauri::command]
pub fn remove_local_model(app: tauri::AppHandle, model_id: String) -> AppResult<LocalModel> {
    manager::remove(&app, &model_id)
}

pub(crate) fn preferred_installed_model_id(app: &tauri::AppHandle) -> AppResult<String> {
    select_preferred_installed_model(&manager::statuses(app)?)
        .map(|model| model.id.clone())
        .ok_or_else(|| {
            AppError::Model(
                "install a local model before starting automatic local transcription".to_owned(),
            )
        })
}

fn select_preferred_installed_model(models: &[LocalModel]) -> Option<&LocalModel> {
    models
        .iter()
        .find(|model| model.installed && model.preset == "balanced")
        .or_else(|| models.iter().find(|model| model.installed))
}

pub use manager::{installed_path, remove_all};
pub use transcriber::{LocalTranscriptionService, transcribe_file};

#[cfg(test)]
mod tests {
    use super::{LocalModel, select_preferred_installed_model};

    fn model(id: &str, preset: &str, installed: bool) -> LocalModel {
        LocalModel {
            id: id.to_owned(),
            preset: preset.to_owned(),
            label: id.to_owned(),
            description: String::new(),
            byte_count: 1,
            installed,
        }
    }

    #[test]
    fn prefers_the_balanced_installed_model() {
        let models = [
            model("fast", "fast", true),
            model("balanced", "balanced", true),
            model("best", "best", true),
        ];

        assert_eq!(
            select_preferred_installed_model(&models).map(|model| model.id.as_str()),
            Some("balanced")
        );
    }

    #[test]
    fn falls_back_to_the_first_installed_model() {
        let models = [model("fast", "fast", false), model("best", "best", true)];

        assert_eq!(
            select_preferred_installed_model(&models).map(|model| model.id.as_str()),
            Some("best")
        );
    }

    #[test]
    fn returns_none_when_no_model_is_installed() {
        let models = [
            model("fast", "fast", false),
            model("balanced", "balanced", false),
        ];

        assert!(select_preferred_installed_model(&models).is_none());
    }
}
