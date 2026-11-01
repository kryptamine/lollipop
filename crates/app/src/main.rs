#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod overlay;
#[cfg(target_os = "macos")]
mod platform;

use std::error::Error;

use lollipop_core::live::{LiveClient, Snapshot, SystemClock, LIVE_CLIENT_URL};
use lollipop_core::overlay::GAME_BUNDLE_ID;
use overlay::Overlay;
use tauri::{App, AppHandle, Emitter, Manager, WebviewWindow};

// Debug builds can point at scripts/mock-live-client.mjs and stand in another app
// for the game; release builds ignore the environment.
fn dev_setting(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|_| cfg!(debug_assertions))
        .unwrap_or_else(|| default.to_string())
}

// Only a closing event loop fails the callback, so polling stops with the app.
async fn publish_live_client(client: LiveClient, app: AppHandle, overlay: Overlay) {
    let mut was_running = false;
    let on_update = move |update: Option<Snapshot>| -> tauri::Result<()> {
        let running = update.is_some();
        if let Some(snapshot) = update {
            app.emit("snapshot", &snapshot)?;
        }
        if running != was_running {
            overlay.set_match_running(running)?;
            was_running = running;
        }
        Ok(())
    };
    let error = client.poll(&SystemClock, on_update).await;
    log::error!("live client polling stopped: {error}");
}

fn cover_primary_display(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        window.set_position(*monitor.position())?;
        window.set_size(*monitor.size())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn keep_focus_away(app: &mut App, window: &WebviewWindow, overlay: &Overlay) -> tauri::Result<()> {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    platform::order_in_hidden(window)?;
    let overlay = overlay.clone();
    platform::watch_frontmost(move |bundle_id| {
        if let Err(error) = overlay.set_frontmost(bundle_id) {
            log::error!("could not apply the frontmost app: {error}");
        }
    });
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let window = app
        .get_webview_window("overlay")
        .ok_or(tauri::Error::WindowNotFound)?;
    let game_bundle_id = dev_setting("LOLLIPOP_GAME_BUNDLE", GAME_BUNDLE_ID);
    let overlay = Overlay::new(window.clone(), &game_bundle_id);
    #[cfg(target_os = "macos")]
    keep_focus_away(app, &window, &overlay)?;
    cover_primary_display(&window)?;
    // Clicks must always reach the game, never the overlay.
    window.set_ignore_cursor_events(true)?;
    let client = LiveClient::new(&dev_setting("LOLLIPOP_LIVE_URL", LIVE_CLIENT_URL))?;
    let publish = publish_live_client(client, app.handle().clone(), overlay);
    tauri::async_runtime::spawn(publish);
    Ok(())
}

#[allow(clippy::exit, reason = "generate_context! expands to process::exit")]
fn main() -> tauri::Result<()> {
    let log = tauri_plugin_log::Builder::default().level(log::LevelFilter::Info);
    let builder = tauri::Builder::default().plugin(log.build());
    // Without this, launching takes focus from League: see ADR 0006.
    #[cfg(target_os = "macos")]
    let builder = builder.activate_ignoring_other_apps(false);

    builder.setup(setup).run(tauri::generate_context!())
}
