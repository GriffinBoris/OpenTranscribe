mod catalog;
mod manager;
mod transcriber;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

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

struct ActiveModelDownload<'a> {
    downloads: &'a Mutex<HashSet<String>>,
    model_id: String,
}

impl<'a> ActiveModelDownload<'a> {
    fn reserve(downloads: &'a Mutex<HashSet<String>>, model_id: String) -> AppResult<Self> {
        if !downloads
            .lock()
            .expect("model download lock poisoned")
            .insert(model_id.clone())
        {
            return Err(AppError::Model(
                "that local model is already downloading".to_owned(),
            ));
        }

        Ok(Self {
            downloads,
            model_id,
        })
    }
}

impl Drop for ActiveModelDownload<'_> {
    fn drop(&mut self) {
        self.downloads
            .lock()
            .expect("model download lock poisoned")
            .remove(&self.model_id);
    }
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
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
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

    ensure_models_idle(&state)?;
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
    let _download = ActiveModelDownload::reserve(&state.active_model_downloads, model_id.clone())?;

    let download_model_id = model_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
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
    .and_then(|result| result)
}

#[tauri::command]
pub fn remove_local_model(
    app: tauri::AppHandle,
    model_id: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> AppResult<LocalModel> {
    let _transition = state
        .dictation_transition
        .lock()
        .expect("dictation transition lock poisoned");
    ensure_models_idle(&state)?;
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
        .or_else(|| {
            models
                .iter()
                .find(|model| model.installed && model.preset != "cleanup")
        })
}

pub use manager::{installed_path, remove_all};
pub use transcriber::LocalTranscriptionService;

fn ensure_models_idle(state: &crate::state::AppState) -> AppResult<()> {
    crate::app_reset::ensure_idle(state)?;
    let mut workers = state.dictation_workers.try_lock().map_err(|_| {
        AppError::Model("Wait for the local runtime to finish before changing models.".to_owned())
    })?;
    workers.speech.unload();
    workers.cleanup.unload();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;

    use super::{ActiveModelDownload, AppError, LocalModel, select_preferred_installed_model};

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
    fn never_selects_text_cleanup_as_a_speech_model() {
        let cleanup = model("s1-mini-q4_k_m", "cleanup", true);
        assert!(select_preferred_installed_model(std::slice::from_ref(&cleanup)).is_none());
        let models = [cleanup, model("fast", "fast", true)];
        assert_eq!(
            select_preferred_installed_model(&models).map(|model| model.id.as_str()),
            Some("fast")
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

    #[test]
    fn reserves_one_download_per_model_and_releases_it_for_retry() {
        let downloads = Mutex::new(HashSet::new());
        let first = ActiveModelDownload::reserve(&downloads, "balanced".to_owned())
            .expect("first download should reserve the model");

        assert!(matches!(
            ActiveModelDownload::reserve(&downloads, "balanced".to_owned()),
            Err(AppError::Model(message)) if message == "that local model is already downloading"
        ));

        drop(first);

        let retry = ActiveModelDownload::reserve(&downloads, "balanced".to_owned())
            .expect("released download should be retryable");
        drop(retry);
        assert!(
            downloads
                .lock()
                .expect("model download lock should remain usable")
                .is_empty()
        );
    }
}
