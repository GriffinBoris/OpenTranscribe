use opentranscribe_domain::AppEvent;
use tauri::ipc::Channel;

use super::AppState;

#[tauri::command]
pub fn subscribe(channel: Channel<AppEvent>, state: tauri::State<'_, AppState>) {
    *state.event_channel.lock().expect("app state lock poisoned") = Some(channel);
}
