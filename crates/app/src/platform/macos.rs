//! The macOS calls, and the only module allowed `unsafe`.
#![allow(
    unsafe_code,
    reason = "window levels, workspace notifications and speech are only reachable through raw Apple framework calls"
)]

mod app_focus;
mod best_voice;
mod frontmost_app;
pub(crate) mod keychain;
mod overlay_window;
mod recap_window;
mod speech;
mod system_locale;

pub(crate) use app_focus::{hide_app, keep_focus_off, run_as_accessory};
pub(crate) use best_voice::voice_installed;
pub(crate) use frontmost_app::watch_frontmost;
pub(crate) use overlay_window::{order_in_hidden, set_shown};
pub(crate) use recap_window::show_without_focus;
pub(crate) use speech::{speak, stop_speaking};
pub(crate) use system_locale::system_locale;
