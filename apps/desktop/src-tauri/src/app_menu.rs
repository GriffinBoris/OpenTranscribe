use opentranscribe_domain::AppEvent;
use tauri::menu::{
    Menu, MenuEvent, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu, SubmenuBuilder,
};
use tauri::{Manager, Wry};

use crate::events::send_event;
use crate::state::AppState;

const IMPORT_MEDIA_ID: &str = "import_media";

pub fn setup(app: &mut tauri::App<Wry>) -> tauri::Result<()> {
    let menu = Menu::default(app.handle())?;
    let import_media = MenuItem::with_id(
        app,
        IMPORT_MEDIA_ID,
        "Import Media…",
        true,
        Some("CmdOrCtrl+O"),
    )?;

    if let Some(file_menu) = find_file_menu(&menu)? {
        let separator = PredefinedMenuItem::separator(app)?;
        file_menu.prepend_items(&[&import_media, &separator])?;
    } else {
        let file_menu = SubmenuBuilder::new(app, "File")
            .item(&import_media)
            .separator()
            .close_window()
            .build()?;
        menu.prepend(&file_menu)?;
    }

    app.set_menu(menu)?;
    Ok(())
}

pub fn handle_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    if event.id().as_ref() != IMPORT_MEDIA_ID {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    send_event(&app.state::<AppState>(), AppEvent::ImportRequested);
}

fn find_file_menu(menu: &Menu<Wry>) -> tauri::Result<Option<Submenu<Wry>>> {
    for item in menu.items()? {
        if let MenuItemKind::Submenu(submenu) = item
            && submenu.text()? == "File"
        {
            return Ok(Some(submenu));
        }
    }

    Ok(None)
}
