use std::path::{Path, PathBuf};

use opentranscribe_domain::AppSettings;
use tauri::Manager;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage::LibraryRepository;

const LIBRARY_DIRECTORIES: [&str; 4] = ["Inbox", "Projects", "Trash", ".opentranscribe"];

pub fn delete_all(app: &tauri::AppHandle, state: &AppState) -> AppResult<()> {
    ensure_idle(state)?;
    let library_path = state
        .repository
        .lock()
        .expect("app state lock poisoned")
        .as_ref()
        .map(|repository| repository.root_path().to_owned());

    state.openai_credentials.remove()?;
    crate::local_models::remove_all(app)?;

    if let Some(library_path) = library_path {
        remove_library_data(&library_path)?;
    }

    remove_app_directories(&[
        resolve_app_path(app.path().app_config_dir())?,
        resolve_app_path(app.path().app_data_dir())?,
        resolve_app_path(app.path().app_cache_dir())?,
    ])?;

    *state.repository.lock().expect("app state lock poisoned") = None;
    *state.settings.lock().expect("app state lock poisoned") = AppSettings::default();
    Ok(())
}

pub fn ensure_idle(state: &AppState) -> AppResult<()> {
    if state.recorder.status().is_some() {
        return Err(AppError::Application(
            "stop the current recording before resetting OpenTranscribe".to_owned(),
        ));
    }

    if crate::dictation::is_active(state) {
        return Err(AppError::Application(
            "wait for dictation to finish before resetting OpenTranscribe".to_owned(),
        ));
    }

    let has_running_jobs = state
        .repository
        .lock()
        .expect("app state lock poisoned")
        .as_ref()
        .map(LibraryRepository::has_running_jobs)
        .transpose()?
        .unwrap_or(false);

    if has_running_jobs {
        return Err(AppError::Application(
            "wait for active processing to finish before resetting OpenTranscribe".to_owned(),
        ));
    }

    if !state
        .active_model_downloads
        .lock()
        .expect("app state lock poisoned")
        .is_empty()
    {
        return Err(AppError::Application(
            "wait for local model downloads to finish before resetting OpenTranscribe".to_owned(),
        ));
    }

    Ok(())
}

fn resolve_app_path(path: Result<PathBuf, tauri::Error>) -> AppResult<PathBuf> {
    path.map_err(|error| AppError::Application(error.to_string()))
}

fn remove_app_directories(directories: &[PathBuf]) -> AppResult<()> {
    for directory in directories {
        if directory.exists() {
            std::fs::remove_dir_all(directory)?;
        }
    }

    Ok(())
}

fn remove_library_data(library_path: &Path) -> AppResult<()> {
    for directory in LIBRARY_DIRECTORIES {
        let target = library_path.join(directory);

        if target.exists() {
            std::fs::remove_dir_all(target)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{remove_app_directories, remove_library_data};

    #[test]
    fn clears_app_owned_directories_without_deleting_the_library() {
        let root = tempdir().expect("temporary directory should be created");
        let config = root.path().join("config");
        let data = root.path().join("data");
        let cache = root.path().join("cache");
        let library = root.path().join("Documents").join("OpenTranscribe");

        for directory in [&config, &data, &cache, &library] {
            fs::create_dir_all(directory).expect("test directory should be created");
            fs::write(directory.join("data"), b"stored").expect("test data should be written");
        }

        remove_app_directories(&[config.clone(), data.clone(), cache.clone()])
            .expect("app data should be removed");

        assert!(!config.exists());
        assert!(!data.exists());
        assert!(!cache.exists());
        assert!(library.join("data").exists());
    }

    #[test]
    fn clears_downloaded_models_without_deleting_the_library_around_them() {
        let root = tempdir().expect("temporary directory should be created");
        let library = root.path().join("Documents").join("OpenTranscribe");
        let models = library.join("models");
        let projects = library.join("Projects");

        for directory in [&models, &projects] {
            fs::create_dir_all(directory).expect("test directory should be created");
            fs::write(directory.join("data"), b"stored").expect("test data should be written");
        }

        remove_app_directories(std::slice::from_ref(&models)).expect("models should be removed");

        assert!(!models.exists());
        assert!(projects.join("data").exists());
    }

    #[test]
    fn accepts_overlapping_platform_directories() {
        let root = tempdir().expect("temporary directory should be created");
        let shared = root.path().join("application-support");
        fs::create_dir_all(&shared).expect("test directory should be created");

        remove_app_directories(&[shared.clone(), shared.clone()])
            .expect("duplicate platform paths should be removed once");

        assert!(!shared.exists());
    }

    #[test]
    fn clears_only_app_owned_library_directories() {
        let root = tempdir().expect("temporary directory should be created");
        let library = root.path().join("library");

        for directory in ["Inbox", "Projects", "Trash", ".opentranscribe"] {
            fs::create_dir_all(library.join(directory))
                .expect("library directory should be created");
        }
        fs::write(library.join("keep.txt"), b"unrelated")
            .expect("unrelated file should be written");

        remove_library_data(&library).expect("library data should be removed");

        assert!(library.join("keep.txt").exists());
        for directory in ["Inbox", "Projects", "Trash", ".opentranscribe"] {
            assert!(!library.join(directory).exists());
        }
    }
}
