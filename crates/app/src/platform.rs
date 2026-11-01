//! OS-specific calls, and the only module allowed `unsafe`.
#![allow(
    unsafe_code,
    reason = "window levels and workspace notifications are only reachable through raw AppKit calls"
)]

mod frontmost_app;
mod overlay_window;

pub(crate) use frontmost_app::watch_frontmost;
pub(crate) use overlay_window::{order_in_hidden, set_shown};
