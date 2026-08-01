use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, Wry};

use crate::commands::{self, AppState};

const TRAY_ID: &str = "opentranscribe";
const OPEN_ID: &str = "open";
const PAUSE_ID: &str = "pause";
const STOP_ID: &str = "stop";
const QUIT_ID: &str = "quit";

pub struct TrayControls {
    pause: MenuItem<Wry>,
    stop: MenuItem<Wry>,
}

pub fn setup(app: &mut tauri::App<Wry>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, OPEN_ID, "Open OpenTranscribe", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, PAUSE_ID, "Pause recording", false, None::<&str>)?;
    let stop = MenuItem::with_id(app, STOP_ID, "Stop recording", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit OpenTranscribe", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &pause, &stop, &quit])?;
    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("OpenTranscribe")
        .icon(tauri::include_image!("icons/tray-icon.png"))
        .icon_as_template(cfg!(target_os = "macos"))
        .on_menu_event(handle_menu_event)
        .build(app)?;
    app.manage(TrayControls {
        pause: pause.clone(),
        stop: stop.clone(),
    });
    Ok(())
}

pub fn set_recording_state(app: &tauri::AppHandle, paused: Option<bool>) {
    let controls = app.state::<TrayControls>();
    let is_recording = paused.is_some();
    let _ = controls.pause.set_enabled(is_recording);
    let _ = controls.stop.set_enabled(is_recording);
    let _ = controls.pause.set_text(if paused == Some(true) {
        "Resume recording"
    } else {
        "Pause recording"
    });

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = match paused {
            Some(true) => "OpenTranscribe — recording paused",
            Some(false) => "OpenTranscribe — recording",
            None => "OpenTranscribe",
        };
        let _ = tray.set_tooltip(Some(tooltip));
        let _ = tray.set_title(paused.map(|is_paused| if is_paused { "Paused" } else { "●" }));
    }
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        OPEN_ID => show_main_window(app),
        PAUSE_ID => toggle_pause(app),
        STOP_ID => stop_recording(app),
        QUIT_ID => {
            if !crate::prevent_exit_while_recording(app) {
                app.exit(0);
            }
        }
        _ => {}
    }
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_pause(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();

    if let Some(status) = commands::recording::recording_status(state)
        && let Err(error) =
            commands::recording::pause_recording(!status.is_paused, app.clone(), app.state())
    {
        log::error!("tray pause failed: {error}");
    }
}

fn stop_recording(app: &tauri::AppHandle) {
    if commands::recording::recording_status(app.state()).is_none() {
        return;
    }

    if let Err(error) = commands::recording::stop_recording(app.clone(), app.state()) {
        log::error!("tray stop failed: {error}");
    }
}
