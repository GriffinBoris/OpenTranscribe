use crate::error::{AppError, AppResult};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub(super) const LABEL: &str = "dictation";

pub(super) fn show(app: &tauri::AppHandle, reposition: bool) -> AppResult<()> {
    let window = match app.get_webview_window(LABEL) {
        Some(window) => window,
        None => {
            WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html?dictation=1".into()))
                .title("Dictation")
                .inner_size(520.0, 220.0)
                .resizable(false)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .focusable(true)
                .visible(false)
                .build()
                .map_err(|error| AppError::Application(error.to_string()))?
        }
    };
    if !reposition
        && window
            .is_visible()
            .map_err(|error| AppError::Application(error.to_string()))?
    {
        return Ok(());
    }
    show_on_current_screen(window)
}

#[cfg(target_os = "macos")]
fn show_on_current_screen(window: tauri::WebviewWindow) -> AppResult<()> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen, NSWindow};

    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let handle = window.clone();
    window
        .run_on_main_thread(move || {
            let result = (|| {
                let pointer = handle
                    .ns_window()
                    .map_err(|error| AppError::Application(error.to_string()))?;
                // Tauri owns this NSWindow; the handle keeps it alive on the AppKit thread.
                let native = unsafe { &*pointer.cast::<NSWindow>() };
                let point = NSEvent::mouseLocation();
                let screens =
                    NSScreen::screens(MainThreadMarker::new().expect("AppKit main thread"));
                let screen = screens
                    .iter()
                    .find(|screen| {
                        let frame = screen.frame();
                        point.x >= frame.origin.x
                            && point.x < frame.origin.x + frame.size.width
                            && point.y >= frame.origin.y
                            && point.y < frame.origin.y + frame.size.height
                    })
                    .ok_or_else(|| {
                        AppError::Application("No display contains the pointer.".to_owned())
                    })?;
                // AppKit uses logical points with a bottom-left origin on every display.
                // Keep pointer, work area and window geometry in that coordinate system.
                let frame = screen.visibleFrame();
                let size = native.frame().size;
                let mut origin = frame.origin;
                origin.x += ((frame.size.width - size.width) / 2.0).max(0.0);
                origin.y += 32.0_f64.min((frame.size.height - size.height).max(0.0));
                native.setFrameOrigin(origin);
                // Tauri's show() calls makeKeyAndOrderFront on macOS, stealing the paste target.
                native.orderFrontRegardless();
                Ok(())
            })();
            let _ = sender.send(result);
        })
        .map_err(|error| AppError::Application(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| AppError::Application(error.to_string()))?
}

#[cfg(not(target_os = "macos"))]
fn show_on_current_screen(window: tauri::WebviewWindow) -> AppResult<()> {
    let point = window
        .cursor_position()
        .map_err(|error| AppError::Application(error.to_string()))?;
    if let Some(monitor) = window
        .monitor_from_point(point.x, point.y)
        .map_err(|error| AppError::Application(error.to_string()))?
    {
        let size = window
            .outer_size()
            .map_err(|error| AppError::Application(error.to_string()))?;
        let origin = monitor.position();
        let extent = monitor.size();
        window
            .set_position(tauri::PhysicalPosition::new(
                origin.x + (extent.width.saturating_sub(size.width) / 2) as i32,
                origin.y + extent.height.saturating_sub(size.height + 80) as i32,
            ))
            .map_err(|error| AppError::Application(error.to_string()))?;
    }
    window
        .show()
        .map_err(|error| AppError::Application(error.to_string()))
}

pub(super) fn hide(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL)
        && let Err(error) = window.hide()
    {
        log::warn!("could not hide dictation panel: {error}");
    }
}
