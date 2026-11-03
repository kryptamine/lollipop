//! OS-specific calls, and the only module allowed `unsafe`.
#![allow(
    unsafe_code,
    reason = "window levels, workspace notifications and the system locale are only reachable through raw AppKit calls"
)]

mod frontmost_app;
mod overlay_window;
mod system_locale;

pub(crate) use frontmost_app::watch_frontmost;
pub(crate) use overlay_window::{order_in_hidden, set_shown};
pub(crate) use system_locale::system_locale;
