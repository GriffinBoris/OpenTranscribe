use crate::error::{AppError, AppResult};
use tauri::{Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub(super) const LABEL: &str = "dictation";

pub(super) fn show(app: &tauri::AppHandle) -> AppResult<()> {
    let window = match app.get_webview_window(LABEL) {
        Some(window) => window,
        None => {
            let window = WebviewWindowBuilder::new(
                app,
                LABEL,
                WebviewUrl::App("index.html?dictation=1".into()),
            )
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
            .map_err(|error| AppError::Application(error.to_string()))?;
            if let Some(main) = app.get_webview_window("main") {
                let monitor = main
                    .cursor_position()
                    .ok()
                    .and_then(|point| main.monitor_from_point(point.x, point.y).ok().flatten());
                if let Some(monitor) = monitor {
                    let size = window
                        .outer_size()
                        .map_err(|error| AppError::Application(error.to_string()))?;
                    let origin = monitor.position();
                    let extent = monitor.size();
                    window
                        .set_position(PhysicalPosition::new(
                            origin.x + (extent.width.saturating_sub(size.width) / 2) as i32,
                            origin.y + extent.height.saturating_sub(size.height + 80) as i32,
                        ))
                        .map_err(|error| AppError::Application(error.to_string()))?;
                }
            }
            window
        }
    };
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
