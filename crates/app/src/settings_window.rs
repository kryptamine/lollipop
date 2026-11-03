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
        .inner_size(520.0, 340.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .theme(Some(Theme::Dark))
        .center()
        .build()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            hand_focus_back(&handle);
        }
    });
    window.set_focus()
}

// Hiding the app is what gives focus back to the app the window was opened over.
// The overlay window opts out of hiding, so it stays.
fn hand_focus_back(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.hide();
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}
