pub(crate) mod application;
pub(crate) mod credentials;
pub(crate) mod events;
pub(crate) mod recording;
pub(crate) mod session;

use std::collections::HashSet;
use std::sync::Mutex;

use opentranscribe_domain::{AppEvent, AppSettings, OpenAiTranscriptionModel, RecordingMode};
use tauri::ipc::Channel;

use crate::audio::RecordingController;
use crate::error::{AppError, AppResult};
use crate::storage::LibraryRepository;
use crate::transcription::OpenAiRealtimeController;

pub(crate) use events::send_event;

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
    pub(crate) recorder: RecordingController,
    pub(crate) active_recording_intent: Mutex<Option<ActiveRecordingIntent>>,
    pub(crate) live_transcription: Mutex<Option<OpenAiRealtimeController>>,
}

pub(crate) fn with_repository<T>(
    state: &tauri::State<'_, AppState>,
    operation: impl FnOnce(&LibraryRepository) -> AppResult<T>,
) -> AppResult<T> {
    let repository = state.repository.lock().expect("app state lock poisoned");
    let repository = repository.as_ref().ok_or(AppError::LibraryNotSelected)?;
    operation(repository)
}
