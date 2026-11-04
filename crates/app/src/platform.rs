//! OS-specific calls. macOS gets the real ones, and any other OS the fallback.

#[cfg(not(target_os = "macos"))]
mod fallback;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
use fallback as os;
#[cfg(target_os = "macos")]
use macos as os;

pub(crate) use os::{
    hide_app, keep_focus_off, keychain, order_in_hidden, run_as_accessory, set_shown, speak,
    stop_speaking, system_locale, voice_installed, watch_frontmost,
};
