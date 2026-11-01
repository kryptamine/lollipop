#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod overlay;
#[cfg(target_os = "macos")]
mod platform;

use lollipop_core::live::{poll, Snapshot, SystemClock, LIVE_CLIENT_URL};
use lollipop_core::overlay::GAME_BUNDLE_ID;
use overlay::Overlay;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

// Debug builds can point at scripts/mock-live-client.mjs and stand in another app
// for the game; release builds ignore the environment.
fn dev_setting(name: &str, release_value: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|_| cfg!(debug_assertions))
        .unwrap_or_else(|| release_value.to_string())
}

#[allow(clippy::expect_used, reason = "tauri.conf.json declares this window")]
fn overlay_window(app: &AppHandle) -> WebviewWindow {
    app.get_webview_window("overlay")
        .expect("overlay window is declared in tauri.conf.json")
}

async fn publish_live_client(app: AppHandle, overlay: Overlay) {
    let on_update = move |update: Option<Snapshot>| {
        let running = update.is_some();
        if let Some(snapshot) = update {
            let _ = app.emit("snapshot", &snapshot);
        }
        overlay.set_match_running(running);
    };
    let url = dev_setting("LOLLIPOP_LIVE_URL", LIVE_CLIENT_URL);
    poll(&url, &SystemClock, on_update).await;
}

#[allow(
    clippy::exit,
    clippy::expect_used,
    reason = "generate_context! expands to process::exit, and the app needs its event loop"
)]
fn main() {
    let builder = tauri::Builder::default();
    // Without this, launching takes focus from League: see ADR 0006.
    #[cfg(target_os = "macos")]
    let builder = builder.activate_ignoring_other_apps(false);

    builder
        .setup(|app| {
            let window = overlay_window(app.handle());
            let game_bundle_id = dev_setting("LOLLIPOP_GAME_BUNDLE", GAME_BUNDLE_ID);
            let overlay = Overlay::new(window.clone(), &game_bundle_id);
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                platform::order_in_hidden(&window)?;
                let frontmost = overlay.clone();
                platform::watch_frontmost(move |bundle_id| frontmost.set_frontmost(bundle_id));
            }
            cover_primary_display(&window)?;
            // Clicks must always reach the game, never the overlay.
            window.set_ignore_cursor_events(true)?;
            tauri::async_runtime::spawn(publish_live_client(app.handle().clone(), overlay));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lollipop");
}

fn cover_primary_display(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        window.set_position(*monitor.position())?;
        window.set_size(*monitor.size())?;
    }
    Ok(())
}
