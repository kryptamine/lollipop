#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
mod platform;

use lollipop_core::live::{poll, Snapshot, SystemClock, LIVE_CLIENT_URL};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

fn live_client_url() -> String {
    // Debug builds can point at scripts/mock-live-client.mjs; release builds cannot.
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("LOLLIPOP_LIVE_URL") {
        return url;
    }
    LIVE_CLIENT_URL.to_string()
}

#[allow(clippy::expect_used, reason = "tauri.conf.json declares this window")]
fn overlay_window(app: &AppHandle) -> WebviewWindow {
    app.get_webview_window("overlay")
        .expect("overlay window is declared in tauri.conf.json")
}

async fn publish_live_client(app: AppHandle) {
    let window = overlay_window(&app);
    let mut shown = false;
    let on_update = move |update: Option<Snapshot>| match update {
        Some(snapshot) => {
            let _ = app.emit("snapshot", &snapshot);
            if !shown {
                let _ = window.show();
                shown = true;
            }
        }
        None if shown => {
            let _ = window.hide();
            shown = false;
        }
        None => {}
    };
    poll(&live_client_url(), &SystemClock, on_update).await;
}

#[allow(
    clippy::exit,
    clippy::expect_used,
    reason = "generate_context! expands to process::exit, and the app needs its event loop"
)]
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = overlay_window(app.handle());
            #[cfg(target_os = "macos")]
            platform::raise_above_shield(&window)?;
            // Clicks must always reach the game, never the overlay.
            window.set_ignore_cursor_events(true)?;
            tauri::async_runtime::spawn(publish_live_client(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lollipop");
}
