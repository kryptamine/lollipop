#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod launch_at_login;
mod menu_bar;
mod overlay;
#[cfg(target_os = "macos")]
mod platform;
mod settings_store;
mod table_downloads;
mod updates;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lollipop_core::delivery::TableFile;
use lollipop_core::game_inputs::LOCKFILE_PATH;
use lollipop_core::live::{poll, status_line, LiveUpdate, SystemClock, LIVE_CLIENT_URL};
use lollipop_core::overlay::GAME_BUNDLE_ID;
use overlay::Overlay;
use settings_store::SettingsStore;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, Wry};
use tauri_plugin_autostart::MacosLauncher;
use updates::Updates;

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

async fn publish_live_client(
    app: AppHandle,
    overlay: Overlay,
    store: SettingsStore,
    status: MenuItem<Wry>,
    benchmark_table: Arc<TableFile>,
) {
    let mut shown_status = status_line(None);
    let on_update = move |update: LiveUpdate| {
        let LiveUpdate {
            game,
            between_games,
        } = update;
        let running = game.is_some();
        // Before the snapshot: the view draws no Widget until it knows its switch.
        store.publish_switches();
        if let Some(game) = &game {
            let _ = app.emit("markers", game.markers);
            let _ = app.emit("benchmarks", game.benchmarks);
            let _ = app.emit("notices", game.shown_notices());
            let _ = app.emit("timing-hints", &game.hints);
            let _ = app.emit("snapshot", &game.snapshot);
        }
        overlay.set_match_running(running);
        if let Some(updates) = app.try_state::<Updates>() {
            updates.set_between_games(between_games);
        }
        let next_status = status_line(game.as_ref());
        if next_status != shown_status {
            let _ = status.set_text(&next_status);
            shown_status = next_status;
        }
    };
    let url = dev_setting("LOLLIPOP_LIVE_URL", LIVE_CLIENT_URL);
    let lockfile = dev_setting("LOLLIPOP_LOCKFILE", LOCKFILE_PATH);
    poll(
        &url,
        Path::new(&lockfile),
        &benchmark_table,
        &SystemClock,
        on_update,
    )
    .await;
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
        // LaunchAgent, as the AppleScript launcher would ask for Automation access.
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            let store = SettingsStore::open(app.handle())?;
            // A login item removed outside the app comes back: the settings file decides.
            let launch_on_login = store.read(|settings| settings.launch_at_login);
            let _ = launch_at_login::apply(app.handle(), launch_on_login);
            let status = menu_bar::install(app.handle(), store.clone())?;
            // A debug build would replace itself with the latest release.
            if !cfg!(debug_assertions) {
                app.manage(Updates::new(app.handle().clone()));
            }
            let delivered = app.path().app_data_dir()?.join("benchmark-table.json");
            let table = dev_setting("LOLLIPOP_BENCHMARK_TABLE", &delivered.to_string_lossy());
            let benchmark_table = Arc::new(TableFile::new(PathBuf::from(table)));
            // A stand-in table from `make dev-mock` must not be overwritten.
            if benchmark_table.path() == delivered {
                tauri::async_runtime::spawn(table_downloads::run(Arc::clone(&benchmark_table)));
            }
            tauri::async_runtime::spawn(publish_live_client(
                app.handle().clone(),
                overlay,
                store,
                status,
                benchmark_table,
            ));
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
