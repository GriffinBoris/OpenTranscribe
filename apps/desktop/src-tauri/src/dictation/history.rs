use crate::error::{AppError, AppResult};
use crate::storage::atomic_file;
use opentranscribe_domain::DictationHistoryEntry;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
const HISTORY_FILENAME: &str = "history.json";
pub(crate) fn history(app: &tauri::AppHandle) -> AppResult<Vec<DictationHistoryEntry>> {
    let path = history_path(app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn clear_history(app: &tauri::AppHandle) -> AppResult<()> {
    clear_history_file(&history_path(app)?)
}

fn clear_history_file(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

pub(super) fn save_history(app: &tauri::AppHandle, entry: DictationHistoryEntry) -> AppResult<()> {
    let path = history_path(app)?;
    let mut entries = history(app)?;
    entries.retain(|previous| previous.id != entry.id);
    entries.insert(0, entry);
    atomic_file::write_json(&path, &entries)
}

fn history_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("dictation");
    fs::create_dir_all(&directory)?;
    Ok(directory.join(HISTORY_FILENAME))
}

pub(super) fn recordings_directory(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("dictation");
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

pub(super) fn remove_recovery_directory(path: &Path, outcome: &str) {
    if let Err(error) = fs::remove_dir_all(path) {
        log::warn!("could not remove {outcome} dictation audio: {error}");
    }
}

pub(crate) fn clear_temporary_recordings(app: &tauri::AppHandle) -> AppResult<()> {
    let root = recordings_directory(app)?;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}
