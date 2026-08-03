use opentranscribe_domain::{DictationHistoryEntry, DictationStatus};
use tauri::ipc::Channel;

use crate::error::AppResult;

use super::AppState;

#[tauri::command]
pub fn toggle_dictation(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<DictationStatus> {
    crate::dictation::toggle(app, &state)
}

#[tauri::command]
pub fn dictation_status(state: tauri::State<'_, AppState>) -> DictationStatus {
    crate::dictation::status(&state)
}

#[tauri::command]
pub fn dictation_shortcut(state: tauri::State<'_, AppState>) -> String {
    state
        .settings
        .lock()
        .expect("app settings lock poisoned")
        .dictation_shortcut
        .0
        .clone()
}

#[tauri::command]
pub fn dictation_history(app: tauri::AppHandle) -> AppResult<Vec<DictationHistoryEntry>> {
    crate::dictation::history(&app)
}

#[tauri::command]
pub fn clear_dictation_history(app: tauri::AppHandle) -> AppResult<()> {
    crate::dictation::clear_history(&app)
}

#[tauri::command]
pub fn cancel_dictation(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<DictationStatus> {
    crate::dictation::cancel(&app, &state)
}

#[tauri::command]
pub fn dismiss_dictation(app: tauri::AppHandle) {
    crate::dictation::dismiss(&app);
}

#[tauri::command]
pub fn subscribe_dictation_status(
    channel: Channel<DictationStatus>,
    state: tauri::State<'_, AppState>,
) {
    *state
        .dictation_status_channel
        .lock()
        .expect("dictation status channel lock poisoned") = Some(channel);
}
