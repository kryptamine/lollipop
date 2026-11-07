use tauri::{AppHandle, Manager, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const LABEL: &str = "recap";

/// Opens the Recap window over the League client without taking focus, or brings the
/// open one forward: see ADR 0024.
pub(crate) fn open(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return crate::platform::show_without_focus(&window);
    }
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("recap.html".into()))
        .title("Recap")
        .inner_size(640.0, 740.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .theme(Some(Theme::Dark))
        .center()
        .visible(false)
        .focused(false)
        .build()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            crate::platform::hide_app(&handle);
        }
    });
    crate::platform::show_without_focus(&window)
}
