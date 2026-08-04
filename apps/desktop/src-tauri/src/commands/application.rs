use std::path::{Path, PathBuf};

use opentranscribe_domain::{
    APP_SETTINGS_SCHEMA_VERSION, AppSettings, AppSnapshot, Appearance, DictationProvider,
    OpenAiTranscriptionModel, Project, RecordingMode, RecordingProjectSelection, SearchFilters,
    SearchPage, Session,
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

const LIBRARY_SELECTION_FILENAME: &str = "selected-library.json";
const LEGACY_LIBRARY_SELECTION_FILENAME: &str = "library.json";

#[derive(Deserialize)]
pub struct SaveSettingsRequest {
    updates: SettingsPatch,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct SettingsPatch {
    setup_completed: Option<bool>,
    recording_mode: Option<RecordingMode>,
    openai_transcription_model: Option<OpenAiTranscriptionModel>,
    #[serde(default)]
    microphone_device_id: PatchValue<Option<String>>,
    capture_system_audio: Option<bool>,
    microphone_echo_cancellation: Option<bool>,
    #[serde(default)]
    local_models_directory: PatchValue<Option<String>>,
    recording_project_selection: Option<RecordingProjectSelection>,
    global_shortcut_enabled: Option<bool>,
    global_shortcut: Option<opentranscribe_domain::GlobalShortcut>,
    dictation_shortcut_enabled: Option<bool>,
    dictation_shortcut: Option<opentranscribe_domain::GlobalShortcut>,
    dictation_provider: Option<DictationProvider>,
    #[serde(default)]
    dictation_local_model_id: PatchValue<Option<String>>,
    dictation_openai_model: Option<OpenAiTranscriptionModel>,
    dictation_auto_paste: Option<bool>,
    appearance: Option<Appearance>,
}

#[derive(Default)]
enum PatchValue<T> {
    #[default]
    Unchanged,
    Set(T),
}

impl<'de, T> Deserialize<'de> for PatchValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Set(T::deserialize(deserializer)?))
    }
}

impl SettingsPatch {
    fn apply(self, settings: &mut AppSettings) {
        if let Some(value) = self.setup_completed {
            settings.setup_completed = value;
        }
        if let Some(value) = self.recording_mode {
            settings.recording_mode = value;
        }
        if let Some(value) = self.openai_transcription_model {
            settings.openai_transcription_model = value;
        }
        if let PatchValue::Set(value) = self.microphone_device_id {
            settings.microphone_device_id = value;
        }
        if let Some(value) = self.capture_system_audio {
            settings.capture_system_audio = value;
        }
        if let Some(value) = self.microphone_echo_cancellation {
            settings.microphone_echo_cancellation = value;
        }
        if let PatchValue::Set(value) = self.local_models_directory {
            settings.local_models_directory = value;
        }
        if let Some(value) = self.recording_project_selection {
            settings.recording_project_selection = value;
        }
        if let Some(value) = self.global_shortcut_enabled {
            settings.global_shortcut_enabled = value;
        }
        if let Some(value) = self.global_shortcut {
            settings.global_shortcut = value;
        }
        if let Some(value) = self.dictation_shortcut_enabled {
            settings.dictation_shortcut_enabled = value;
        }
        if let Some(value) = self.dictation_shortcut {
            settings.dictation_shortcut = value;
        }
        if let Some(value) = self.dictation_provider {
            settings.dictation_provider = value;
        }
        if let PatchValue::Set(value) = self.dictation_local_model_id {
            settings.dictation_local_model_id = value;
        }
        if let Some(value) = self.dictation_openai_model {
            settings.dictation_openai_model = value;
        }
        if let Some(value) = self.dictation_auto_paste {
            settings.dictation_auto_paste = value;
        }
        if let Some(value) = self.appearance {
            settings.appearance = value;
        }
    }
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
    request: SaveSettingsRequest,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<AppSettings> {
    update_settings(&app, &state, |settings| request.updates.apply(settings))
}

#[tauri::command]
pub fn reset_application_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<AppSettings> {
    crate::app_reset::ensure_idle(&state)?;
    update_settings(&app, &state, |settings| {
        let local_models_directory = settings.local_models_directory.clone();
        *settings = AppSettings {
            local_models_directory,
            ..AppSettings::default()
        };
    })
}

#[tauri::command]
pub fn delete_all_application_data(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<()> {
    crate::app_reset::delete_all(&app, &state)?;
    app.restart()
}

#[tauri::command]
pub fn updater_configured() -> bool {
    updater_public_key().is_some()
}

pub(crate) fn updater_public_key() -> Option<&'static str> {
    let public_key = option_env!("OPENTRANSCRIBE_UPDATER_PUBLIC_KEY")
        .filter(|public_key| !public_key.trim().is_empty())?;

    #[cfg(target_os = "linux")]
    std::env::var_os("APPIMAGE")?;

    Some(public_key)
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
pub async fn import_media(path: String, app: tauri::AppHandle) -> AppResult<Session> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        with_repository(&state, |repository| {
            repository.import_media(&PathBuf::from(path))
        })
    })
    .await
    .map_err(|error| AppError::Application(format!("media import task failed: {error}")))?
}

fn remembered_library(app: &tauri::AppHandle) -> AppResult<Option<PathBuf>> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Application(error.to_string()))?;
    load_remembered_library(&directory)
}

fn load_remembered_library(directory: &Path) -> AppResult<Option<PathBuf>> {
    let path = directory.join(LIBRARY_SELECTION_FILENAME);

    if path.exists() {
        let selection: LibrarySelection = serde_json::from_slice(&std::fs::read(path)?)?;
        return Ok(Some(selection.path));
    }

    let legacy_path = directory.join(LEGACY_LIBRARY_SELECTION_FILENAME);

    if !legacy_path.exists() {
        return Ok(None);
    }

    let selection: LibrarySelection = serde_json::from_slice(&std::fs::read(&legacy_path)?)?;
    crate::storage::atomic_file::write_json(&path, &selection)?;
    std::fs::remove_file(legacy_path)?;
    Ok(Some(selection.path))
}

fn default_library_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    app.path()
        .document_dir()
        .map(|directory| directory.join("OpenTranscribe"))
        .map_err(|error| AppError::Application(error.to_string()))
}

pub(crate) fn load_settings(app: &tauri::AppHandle) -> AppResult<AppSettings> {
    let path = settings_path(app)?;
    if path.exists() {
        let settings = read_settings_file(&path)?;

        if settings.schema_version > APP_SETTINGS_SCHEMA_VERSION {
            return Err(AppError::Application(
                "this settings file was created by a newer OpenTranscribe version".to_owned(),
            ));
        }

        return Ok(settings);
    }

    Ok(AppSettings::default())
}

fn read_settings_file(path: &Path) -> AppResult<AppSettings> {
    let mut stored_settings: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    normalize_stored_global_shortcut(&mut stored_settings);
    Ok(serde_json::from_value(stored_settings)?)
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
        &config_directory.join(LIBRARY_SELECTION_FILENAME),
        &LibrarySelection {
            path: library_path.to_owned(),
        },
    )
}

pub(crate) fn update_settings(
    app: &tauri::AppHandle,
    state: &AppState,
    update: impl FnOnce(&mut AppSettings),
) -> AppResult<AppSettings> {
    let mut current = state.settings.lock().expect("app state lock poisoned");
    let mut settings = current.clone();
    update(&mut settings);
    settings.schema_version = APP_SETTINGS_SCHEMA_VERSION;
    settings.revision = current.revision + 1;
    write_settings_file(&settings_path(app)?, &settings)?;
    *current = settings.clone();
    Ok(settings)
}

fn write_settings_file(path: &Path, settings: &AppSettings) -> AppResult<()> {
    let mut stored_settings = serde_json::to_value(settings)?;
    normalize_stored_global_shortcut(&mut stored_settings);
    crate::storage::atomic_file::write_json(path, &stored_settings)
}

fn normalize_stored_global_shortcut(settings: &mut serde_json::Value) {
    let Some(serde_json::Value::String(shortcut)) = settings.get_mut("global_shortcut") else {
        return;
    };

    let legacy_shortcut = match shortcut.as_str() {
        "CommandOrControl+Shift+R" => "command_or_control_shift_r",
        "CommandOrControl+Shift+Space" => "command_or_control_shift_space",
        "Alt+Shift+R" => "alt_shift_r",
        _ => return,
    };
    *shortcut = legacy_shortcut.to_owned();
}

#[cfg(test)]
mod tests {
    use std::fs;

    use opentranscribe_domain::{AppSettings, GlobalShortcut, RecordingMode};
    use tempfile::tempdir;

    use super::{
        LibrarySelection, SettingsPatch, load_remembered_library, read_settings_file,
        write_settings_file,
    };

    #[test]
    fn settings_patch_updates_only_its_provided_fields() {
        let patch: SettingsPatch = serde_json::from_str(
            r#"{
                "recording_mode": "local_after_recording",
                "microphone_device_id": null,
                "capture_system_audio": true,
                "microphone_echo_cancellation": true,
                "dictation_shortcut": "Alt+Shift+D"
            }"#,
        )
        .expect("settings patch should deserialize");
        let mut settings = AppSettings {
            microphone_device_id: Some("microphone-id".to_owned()),
            setup_completed: true,
            ..AppSettings::default()
        };

        patch.apply(&mut settings);

        assert_eq!(settings.recording_mode, RecordingMode::LocalAfterRecording);
        assert_eq!(settings.microphone_device_id, None);
        assert!(settings.capture_system_audio);
        assert!(settings.microphone_echo_cancellation);
        assert!(settings.setup_completed);
        assert_eq!(settings.dictation_shortcut.0, "Alt+Shift+D");
    }

    #[test]
    fn migrates_the_legacy_remembered_library_file() {
        let directory = tempdir().expect("temporary config directory should be created");
        let library_path = directory.path().join("library");
        let legacy_path = directory.path().join("library.json");
        fs::write(
            &legacy_path,
            serde_json::to_vec(&LibrarySelection {
                path: library_path.clone(),
            })
            .expect("legacy selection should serialize"),
        )
        .expect("legacy selection should be written");

        let selected =
            load_remembered_library(directory.path()).expect("legacy selection should migrate");

        assert_eq!(selected, Some(library_path));
        assert!(!legacy_path.exists());
        assert!(directory.path().join("selected-library.json").exists());
    }

    #[test]
    fn saves_and_loads_a_custom_dictation_shortcut() {
        let directory = tempdir().expect("temporary config directory should be created");
        let path = directory.path().join("settings.json");
        let settings = AppSettings {
            global_shortcut: GlobalShortcut("CommandOrControl+Shift+R".to_owned()),
            dictation_shortcut: GlobalShortcut("Alt+Shift+D".to_owned()),
            ..AppSettings::default()
        };

        write_settings_file(&path, &settings).expect("settings should be written");
        let stored_settings: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("settings should be readable"))
                .expect("settings should be valid JSON");
        let loaded_settings = read_settings_file(&path).expect("settings should be loaded");

        assert_eq!(
            stored_settings["global_shortcut"],
            "command_or_control_shift_r"
        );
        assert_eq!(
            loaded_settings.global_shortcut.0,
            "CommandOrControl+Shift+R"
        );
        assert_eq!(loaded_settings.dictation_shortcut.0, "Alt+Shift+D");
    }
}
