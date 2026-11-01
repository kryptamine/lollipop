use lollipop_core::live::status_line;
use lollipop_core::settings::{Settings, WIDGETS};
use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{include_image, AppHandle};

use crate::launch_at_login;
use crate::settings_store::SettingsStore;

const LAUNCH_AT_LOGIN_ID: &str = "launch-at-login";
const QUIT_ID: &str = "quit";

/// Builds the menu bar menu, the only settings surface, and returns its status line.
pub(crate) fn install(
    app: &AppHandle,
    store: SettingsStore,
) -> tauri::Result<MenuItem<tauri::Wry>> {
    let status = MenuItem::new(app, status_line(None), false, None::<&str>)?;
    let widgets = store.read(|settings| widget_items(app, settings))?;
    let launch_at_login = CheckMenuItem::with_id(
        app,
        LAUNCH_AT_LOGIN_ID,
        "Launch at login",
        true,
        store.read(|settings| settings.launch_at_login),
        None::<&str>,
    )?;
    let menu = build_menu(app, &status, &widgets, &launch_at_login)?;

    TrayIconBuilder::new()
        .icon(include_image!("icons/icon.png"))
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(move |app, event| {
            handle_click(app, &event, &store, &widgets, &launch_at_login);
        })
        .build(app)?;
    Ok(status)
}

fn widget_items(
    app: &AppHandle,
    settings: &Settings,
) -> tauri::Result<Vec<CheckMenuItem<tauri::Wry>>> {
    WIDGETS
        .iter()
        .map(|widget| {
            CheckMenuItem::with_id(
                app,
                widget.id,
                widget.label,
                true,
                settings.widget_on(widget.id),
                None::<&str>,
            )
        })
        .collect()
}

fn build_menu(
    app: &AppHandle,
    status: &MenuItem<tauri::Wry>,
    widgets: &[CheckMenuItem<tauri::Wry>],
    launch_at_login: &CheckMenuItem<tauri::Wry>,
) -> tauri::Result<Menu<tauri::Wry>> {
    let mut menu = MenuBuilder::new(app).item(status).separator();
    for widget in widgets {
        menu = menu.item(widget);
    }
    menu.separator()
        .item(launch_at_login)
        .separator()
        .text(QUIT_ID, "Quit")
        .build()
}

fn handle_click(
    app: &AppHandle,
    event: &MenuEvent,
    store: &SettingsStore,
    widgets: &[CheckMenuItem<tauri::Wry>],
    launch_at_login: &CheckMenuItem<tauri::Wry>,
) {
    let id = event.id().as_ref();
    if id == QUIT_ID {
        app.exit(0);
    } else if id == LAUNCH_AT_LOGIN_ID {
        switch_launch_at_login(app, store, launch_at_login);
    } else if let Some(item) = widgets.iter().find(|item| item.id() == id) {
        // The menu has already flipped the check mark by the time the click arrives.
        let on = item.is_checked().unwrap_or(true);
        store.change(|settings| settings.set_widget(id, on));
    }
}

fn switch_launch_at_login(
    app: &AppHandle,
    store: &SettingsStore,
    item: &CheckMenuItem<tauri::Wry>,
) {
    let on = item.is_checked().unwrap_or(false);
    if launch_at_login::apply(app, on).is_ok() {
        store.change(|settings| settings.launch_at_login = on);
    } else {
        let _ = item.set_checked(!on);
    }
}
