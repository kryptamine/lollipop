use lollipop_core::live::status_line;
use tauri::menu::{MenuBuilder, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{include_image, AppHandle};

use crate::settings_window;

const SETTINGS_ID: &str = "settings";
const QUIT_ID: &str = "quit";

/// Builds the menu bar menu and returns its status line. Every switch lives in the
/// settings window.
pub(crate) fn install(app: &AppHandle) -> tauri::Result<MenuItem<tauri::Wry>> {
    let status = MenuItem::new(app, status_line(None), false, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings…", true, Some("CmdOrCtrl+,"))?;
    let menu = MenuBuilder::new(app)
        .item(&status)
        .separator()
        .item(&settings)
        .separator()
        .text(QUIT_ID, "Quit")
        .build()?;

    TrayIconBuilder::new()
        .icon(include_image!("icons/icon.png"))
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, event| handle_click(app, &event))
        .build(app)?;
    Ok(status)
}

fn handle_click(app: &AppHandle, event: &MenuEvent) {
    let id = event.id().as_ref();
    if id == QUIT_ID {
        app.exit(0);
    } else if id == SETTINGS_ID {
        let _ = settings_window::open(app);
    }
}
