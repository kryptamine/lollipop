use tauri::AppHandle;
use tauri_plugin_autostart::{Error, ManagerExt};

/// Touches the OS only on a difference, so a `make dev` run cannot repoint the login
/// item at its debug binary.
pub(crate) fn apply(app: &AppHandle, on: bool) -> Result<(), Error> {
    let login_item = app.autolaunch();
    match (on, login_item.is_enabled()?) {
        (true, false) => login_item.enable(),
        (false, true) => login_item.disable(),
        _ => Ok(()),
    }
}
