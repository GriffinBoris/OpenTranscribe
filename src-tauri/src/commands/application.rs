use std::path::{Path, PathBuf};

use opentranscribe_domain::{
    AppSettings, AppSnapshot, Project, SearchFilters, SearchPage, Session,
};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::{AppState, with_repository};
use crate::error::{AppError, AppResult};
use crate::storage::LibraryRepository;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SearchLibraryRequest {
    pub query: String,
    pub filters: SearchFilters,
}

#[derive(Deserialize, Serialize)]
struct LibrarySelection {
    path: PathBuf,
}

#[tauri::command]
pub fn bootstrap(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<AppSnapshot> {
    let settings = load_settings(&app)?;
    *state.settings.lock().expect("app state lock poisoned") = settings.clone();
    let mut repository = state.repository.lock().expect("app state lock poisoned");

    if repository.is_none() {
        let (path, remember_path) = match remembered_library(&app)? {
            Some(path) => (path, false),
            None => (default_library_path(&app)?, true),
        };
        let initialized_repository = LibraryRepository::initialize(&path)?;

        if remember_path {
            remember_library(&app, &path)?;
        }

        *repository = Some(initialized_repository);
    }

    let Some(repository) = repository.as_ref() else {
        return Ok(AppSnapshot {
            library: None,
            projects: Vec::new(),
            recent_sessions: Vec::new(),
            active_jobs: Vec::new(),
            settings,
        });
    };

    repository.refresh_search_index()?;

    Ok(AppSnapshot {
        library: Some(repository.descriptor()?),
        projects: repository.projects()?,
        recent_sessions: repository.sessions()?,
        active_jobs: repository.jobs()?,
        settings,
    })
}

#[tauri::command]
pub fn save_settings(
    mut settings: AppSettings,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<AppSettings> {
    let current_revision = state
        .settings
        .lock()
        .expect("app state lock poisoned")
        .revision;
    settings.revision = current_revision + 1;
    crate::storage::atomic_file::write_json(&settings_path(&app)?, &settings)?;
    *state.settings.lock().expect("app state lock poisoned") = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub fn initialize_library(
    path: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<AppSnapshot> {
    if state.recorder.status().is_some() {
        return Err(AppError::Application(
            "stop the current recording before switching libraries".to_owned(),
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
            "wait for active processing to finish before switching libraries".to_owned(),
        ));
    }

    let path = PathBuf::from(path);
    let repository = LibraryRepository::initialize(&path)?;
    remember_library(&app, &path)?;
    *state.repository.lock().expect("app state lock poisoned") = Some(repository);
    bootstrap(app, state)
}

#[tauri::command]
pub fn create_project(
    request: CreateProjectRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<Project> {
    with_repository(&state, |repository| repository.create_project(request.name))
}

#[tauri::command]
pub fn search_library(
    request: SearchLibraryRequest,
    state: tauri::State<'_, AppState>,
) -> AppResult<SearchPage> {
    with_repository(&state, |repository| {
        repository.search(&request.query, &request.filters)
    })
}

#[tauri::command]
pub fn import_media(path: String, state: tauri::State<'_, AppState>) -> AppResult<Session> {
    with_repository(&state, |repository| {
        repository.import_media(&PathBuf::from(path))
    })
}

fn remembered_library(app: &tauri::AppHandle) -> AppResult<Option<PathBuf>> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Application(error.to_string()))?
        .join("library.json");

    if !path.exists() {
        return Ok(None);
    }

    let selection: LibrarySelection = serde_json::from_slice(&std::fs::read(path)?)?;
    Ok(Some(selection.path))
}

fn default_library_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    app.path()
        .document_dir()
        .map(|directory| directory.join("OpenTranscribe"))
        .map_err(|error| AppError::Application(error.to_string()))
}

fn load_settings(app: &tauri::AppHandle) -> AppResult<AppSettings> {
    let path = settings_path(app)?;
    if path.exists() {
        return Ok(serde_json::from_slice(&std::fs::read(path)?)?);
    }

    Ok(AppSettings::default())
}

fn settings_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Application(error.to_string()))?;
    std::fs::create_dir_all(&directory)?;
    Ok(directory.join("settings.json"))
}

fn remember_library(app: &tauri::AppHandle, library_path: &Path) -> AppResult<()> {
    let config_directory = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Application(error.to_string()))?;
    std::fs::create_dir_all(&config_directory)?;
    crate::storage::atomic_file::write_json(
        &config_directory.join("library.json"),
        &LibrarySelection {
            path: library_path.to_owned(),
        },
    )
}
