#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod live;

use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const TARGET_CS_PER_MIN: f64 = 7.0;

fn live_client_url() -> String {
    // Debug builds can point at scripts/mock-live-client.mjs; release builds cannot.
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var("LOLLIPOP_LIVE_URL") {
        return url;
    }
    LIVE_CLIENT_URL.to_string()
}

async fn poll_live_client(app: AppHandle) {
    let url = live_client_url();
    // The League client serves loopback HTTPS with a self-signed certificate,
    // and this client only ever talks to the fixed loopback URL above.
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(800))
        .build()
        .expect("HTTP client should build");

    let window = app
        .get_webview_window("overlay")
        .expect("overlay window is declared in tauri.conf.json");
    let mut shown = false;

    loop {
        let snapshot = match client.get(&url).send().await {
            Ok(response) => response
                .text()
                .await
                .ok()
                .and_then(|body| live::parse_snapshot(&body, TARGET_CS_PER_MIN)),
            Err(_) => None,
        };

        match snapshot {
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
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("overlay")
                .expect("overlay window is declared in tauri.conf.json");
            // Clicks must always reach the game, never the overlay.
            window.set_ignore_cursor_events(true)?;
            tauri::async_runtime::spawn(poll_live_client(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lollipop");
}
