//! The platform calls for any OS but macOS. Each does nothing but the keychain.
#![allow(
    clippy::missing_const_for_fn,
    clippy::unnecessary_wraps,
    reason = "each call keeps the signature of its macOS twin"
)]

use tauri::{App, AppHandle, Builder, WebviewWindow, Wry};

pub(crate) mod keychain;

pub(crate) fn keep_focus_off(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}

pub(crate) fn run_as_accessory(_: &mut App) {}

pub(crate) fn hide_app(_: &AppHandle) {}

pub(crate) fn order_in_hidden(_: &WebviewWindow) -> tauri::Result<()> {
    Ok(())
}

pub(crate) fn set_shown(_: &WebviewWindow, _: bool) -> tauri::Result<()> {
    Ok(())
}

pub(crate) fn watch_frontmost(_: impl Fn(Option<String>) + 'static) {}

pub(crate) fn system_locale() -> Option<String> {
    None
}

pub(crate) fn voice_installed(_: &str) -> bool {
    false
}

pub(crate) fn speak(_: &str, _: &str, _: f32) {}

pub(crate) fn stop_speaking() {}
