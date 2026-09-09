use crate::error::{AppError, AppResult};
use crate::state::AppState;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::Manager;

#[cfg(target_os = "macos")]
pub(super) fn foreground_target() -> Option<u64> {
    objc2_app_kit::NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .and_then(|app| u64::try_from(app.processIdentifier()).ok())
        .filter(|pid| *pid > 0 && *pid != u64::from(std::process::id()))
}

#[cfg(target_os = "windows")]
pub(super) fn foreground_target() -> Option<u64> {
    let window = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
    (!window.0.is_null()).then_some(window.0 as u64)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn foreground_target() -> Option<u64> {
    None
}

pub(super) async fn deliver(
    app: &tauri::AppHandle,
    text: String,
    auto_paste: bool,
    target: Option<u64>,
    canceled: Arc<AtomicBool>,
) -> AppResult<bool> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let state = handle.state::<AppState>();
        // Serialize cancellation with the final side effect, not just with inference completion.
        let _transition = state
            .dictation_transition
            .lock()
            .expect("dictation transition lock poisoned");
        let result = if canceled.load(Ordering::SeqCst) {
            Err(AppError::JobCanceled)
        } else {
            deliver_text(&text, auto_paste, target)
        };
        let _ = sender.send(result);
    })
    .map_err(|error| AppError::Application(error.to_string()))?;
    receiver
        .await
        .map_err(|_| AppError::Application("Dictation delivery was interrupted.".to_owned()))?
}

fn deliver_text(text: &str, auto_paste: bool, target: Option<u64>) -> AppResult<bool> {
    let mut clipboard =
        Clipboard::new().map_err(|error| AppError::Application(error.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|error| AppError::Application(error.to_string()))?;
    if !should_paste(auto_paste, target, foreground_target()) {
        return Ok(false);
    }
    match paste(target) {
        Ok(pasted) => Ok(pasted),
        Err(error) => {
            log::warn!("dictation copied; automatic paste failed: {error}");
            Ok(false)
        }
    }
}

fn should_paste(enabled: bool, original: Option<u64>, current: Option<u64>) -> bool {
    enabled && original.is_some() && original == current
}

fn paste(target: Option<u64>) -> Result<bool, String> {
    let mut input = Enigo::new(&Settings::default()).map_err(|error| error.to_string())?;
    if !should_paste(true, target, foreground_target()) {
        return Ok(false);
    }
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    input
        .key(modifier, Direction::Press)
        .map_err(|error| error.to_string())?;
    let click = input.key(Key::Unicode('v'), Direction::Click);
    let release = input.key(modifier, Direction::Release);
    click
        .and(release)
        .map(|()| true)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::should_paste;
    #[test]
    fn copies_instead_of_pasting_after_focus_changes_or_when_target_is_unknown() {
        assert!(should_paste(true, Some(10), Some(10)));
        assert!(!should_paste(true, Some(10), Some(20)));
        assert!(!should_paste(true, None, None));
        assert!(!should_paste(false, Some(10), Some(10)));
    }
}
