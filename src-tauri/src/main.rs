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

// Throwaway: LOLLIPOP_PROBE picks the macOS window setup under test over League.
#[cfg(target_os = "macos")]
fn apply_probe_variant(app: &mut tauri::App, window: &tauri::WebviewWindow) -> tauri::Result<()> {
    use objc2::{msg_send, runtime::AnyObject};

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGShieldingWindowLevel() -> i32;
    }
    const FULL_SCREEN_AUXILIARY: usize = 1 << 8;

    let variant = std::env::var("LOLLIPOP_PROBE").unwrap_or_default();
    if variant.starts_with("accessory") {
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
    let ns_window = window.ns_window()? as *mut AnyObject;
    unsafe {
        if variant.starts_with("accessory-aux") {
            let current: usize = msg_send![ns_window, collectionBehavior];
            let _: () =
                msg_send![ns_window, setCollectionBehavior: current | FULL_SCREEN_AUXILIARY];
        }
        if variant.ends_with("shield") {
            let level = CGShieldingWindowLevel() as isize + 1;
            let _: () = msg_send![ns_window, setLevel: level];
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("overlay")
                .expect("overlay window is declared in tauri.conf.json");
            #[cfg(target_os = "macos")]
            apply_probe_variant(app, &window)?;
            // Clicks must always reach the game, never the overlay.
            window.set_ignore_cursor_events(true)?;
            tauri::async_runtime::spawn(poll_live_client(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lollipop");
}
