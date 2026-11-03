//! OS-specific calls, and the only module allowed `unsafe`.
#![allow(
    unsafe_code,
    reason = "window levels, workspace notifications, the system locale and speech are only reachable through raw Apple framework calls"
)]

mod best_voice;
mod frontmost_app;
mod overlay_window;
mod speech;
mod system_locale;

pub(crate) use best_voice::best_voice;
pub(crate) use frontmost_app::watch_frontmost;
pub(crate) use overlay_window::{order_in_hidden, set_shown};
pub(crate) use speech::{speak, stop_speaking};
pub(crate) use system_locale::system_locale;
