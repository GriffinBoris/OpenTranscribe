use std::collections::HashSet;
use std::sync::Mutex;

use opentranscribe_domain::{AppEvent, AppSettings, OpenAiTranscriptionModel, RecordingMode};
use tauri::ipc::Channel;

use crate::audio::RecordingController;
use crate::credentials::OpenAiCredentials;
use crate::error::{AppError, AppResult};
use crate::storage::LibraryRepository;
use crate::transcription::OpenAiRealtimeController;

pub(crate) struct ActiveRecordingIntent {
    pub(crate) session_id: String,
    pub(crate) mode: RecordingMode,
    pub(crate) openai_model: OpenAiTranscriptionModel,
}

#[derive(Default)]
pub struct AppState {
    pub(crate) repository: Mutex<Option<LibraryRepository>>,
    pub(crate) event_channel: Mutex<Option<Channel<AppEvent>>>,
    pub(crate) settings: Mutex<AppSettings>,
    pub(crate) canceled_jobs: Mutex<HashSet<String>>,
    pub(crate) active_model_downloads: Mutex<HashSet<String>>,
    pub(crate) recorder: RecordingController,
    pub(crate) openai_credentials: OpenAiCredentials,
    pub(crate) active_recording_intent: Mutex<Option<ActiveRecordingIntent>>,
    pub(crate) live_transcription: Mutex<Option<OpenAiRealtimeController>>,
    pub(crate) dictation: Mutex<crate::dictation::DictationRun>,
    pub(crate) dictation_workers: tokio::sync::Mutex<crate::dictation::workers::DictationWorkers>,
    pub(crate) dictation_transition: Mutex<()>,
    pub(crate) dictation_status_channel:
        Mutex<Option<Channel<opentranscribe_domain::DictationStatus>>>,
    #[cfg(target_os = "macos")]
    pub(crate) hide_main_window_after_fullscreen_exit: Mutex<bool>,
}

pub(crate) fn with_repository<T>(
    state: &AppState,
    operation: impl FnOnce(&LibraryRepository) -> AppResult<T>,
) -> AppResult<T> {
    let repository = state.repository.lock().expect("app state lock poisoned");
    let repository = repository.as_ref().ok_or(AppError::LibraryNotSelected)?;
    operation(repository)
}
