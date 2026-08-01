mod app_menu;
mod app_reset;
mod audio;
mod commands;
mod credentials;
mod error;
mod events;
mod exports;
mod jobs;
mod local_models;
mod recording;
mod state;
mod storage;
mod transcription;
mod tray;

use events::send_event;
use opentranscribe_domain::AppEvent;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(|app| {
            app_menu::setup(app)?;
            tray::setup(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();

                if let Err(error) = window.hide() {
                    log::error!("failed to hide the main window: {error}");
                }
            }
        })
        .on_menu_event(app_menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            commands::application::bootstrap,
            commands::application::save_settings,
            commands::application::reset_application_settings,
            commands::application::delete_all_application_data,
            commands::application::initialize_library,
            commands::application::create_project,
            commands::application::search_library,
            commands::recording::create_recording,
            commands::application::import_media,
            commands::session::save_notes,
            commands::session::session_workspace,
            commands::recording::audio_devices,
            commands::recording::open_system_audio_permission_settings,
            commands::recording::pause_recording,
            commands::recording::recording_status,
            commands::recording::stop_recording,
            commands::session::session_audio_sources,
            commands::session::session_waveform,
            commands::session::recover_recording,
            commands::session::reveal_session,
            commands::session::rename_session,
            commands::session::move_session,
            commands::session::trashed_sessions,
            commands::session::trash_session,
            commands::session::restore_session,
            commands::session::empty_trash,
            commands::credentials::openai_credential_status,
            commands::credentials::save_openai_api_key,
            commands::credentials::remove_openai_api_key,
            commands::credentials::test_openai_connection,
            commands::jobs::enqueue_transcription,
            commands::jobs::retry_job,
            commands::jobs::cancel_job,
            commands::session::update_transcript_segment,
            commands::session::rename_speaker,
            commands::session::merge_speakers,
            local_models::local_model_statuses,
            local_models::local_model_storage_path,
            local_models::move_local_models,
            local_models::download_local_model,
            local_models::remove_local_model,
            commands::session::export_session,
            commands::events::subscribe,
        ])
        .build(tauri::generate_context!())
        .expect("OpenTranscribe failed to start");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } = event
            && prevent_exit_while_recording(app_handle)
        {
            api.prevent_exit();
        }
    });
}

pub(crate) fn prevent_exit_while_recording(app: &tauri::AppHandle) -> bool {
    if app.state::<AppState>().recorder.status().is_none() {
        return false;
    }

    let message = "Stop the current recording before quitting OpenTranscribe.".to_owned();
    log::warn!("{message}");
    tray::show_main_window(app);
    send_event(
        &app.state::<AppState>(),
        AppEvent::AttentionRequired(message),
    );
    true
}
