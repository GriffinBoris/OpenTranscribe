use opentranscribe_domain::{AppEvent, LevelSnapshot};
use tauri::Manager;
use tauri::ipc::Channel;

use super::AppState;

#[tauri::command]
pub fn subscribe(channel: Channel<AppEvent>, state: tauri::State<'_, AppState>) {
    *state.event_channel.lock().expect("app state lock poisoned") = Some(channel);
}

pub(crate) fn publish_recording_levels(app: tauri::AppHandle, session_id: String) {
    std::thread::Builder::new()
        .name("recording-level-events".to_owned())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let state = app.state::<AppState>();
                let Some(status) = state.recorder.status() else {
                    return;
                };

                if status.session_id != session_id {
                    return;
                }

                let event = AppEvent::RecordingLevels(LevelSnapshot {
                    session_id: status.session_id,
                    is_paused: status.is_paused,
                    captures_system_audio: status.captures_system_audio,
                    microphone_peak: status.microphone_peak,
                    system_peak: status.system_peak,
                    elapsed_ms: status.elapsed_ms,
                    dropped_packets: status.dropped_packets,
                });

                if !send_event(&state, event) {
                    return;
                }
            }
        })
        .expect("recording level event thread must start");
}

pub(crate) fn send_event(state: &AppState, event: AppEvent) -> bool {
    let channel = state
        .event_channel
        .lock()
        .expect("app state lock poisoned")
        .clone();
    let Some(channel) = channel else {
        return true;
    };

    if let Err(error) = channel.send(event) {
        log::warn!("app event subscriber disconnected: {error}");
        return false;
    }

    true
}
