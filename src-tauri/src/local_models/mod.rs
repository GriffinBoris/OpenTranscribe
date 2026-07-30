mod catalog;
mod manager;
mod transcriber;

use opentranscribe_domain::{JobProgress, JobStage, ProgressUnit};
use serde::Serialize;
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

#[tauri::command]
pub fn local_model_statuses(app: tauri::AppHandle) -> AppResult<Vec<LocalModel>> {
    manager::statuses(&app)
}

#[tauri::command]
pub async fn download_local_model(
    app: tauri::AppHandle,
    model_id: String,
    progress_channel: Channel<JobProgress>,
) -> AppResult<LocalModel> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut progress_connected = true;

        manager::download(&app, &model_id, |completed, total| {
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
    .map_err(|error| AppError::Model(error.to_string()))?
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

pub use manager::installed_path;
pub use transcriber::LocalTranscriptionService;

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
