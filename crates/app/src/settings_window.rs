use tauri::{AppHandle, Manager, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const LABEL: &str = "settings";

/// Opens the settings window, or raises the one already open. Either way Lollipop
/// becomes the active app, which hides the overlay: see ADR 0014.
pub(crate) fn open(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return window.set_focus();
    }
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
        .title("Settings")
        // Fits the Advice tab at its tallest, with the off banner and a key failure: 554px.
        .inner_size(600.0, 560.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .theme(Some(Theme::Dark))
        .center()
        .build()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            crate::platform::hide_app(&handle);
        }
    });
    window.set_focus()
}
