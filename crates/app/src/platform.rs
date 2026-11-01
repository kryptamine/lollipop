//! OS-specific calls, and the only module allowed `unsafe`.
#![allow(
    unsafe_code,
    reason = "macOS window levels are only reachable through raw AppKit calls"
)]

use objc2::{msg_send, runtime::AnyObject};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGShieldingWindowLevel() -> i32;
}

/// League's Fullscreen windows sit at the shield level, so the overlay goes one above it.
pub fn raise_above_shield(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let ns_window = window.ns_window()?.cast::<AnyObject>();
    // SAFETY: `ns_window` is the live NSWindow Tauri owns, and setup runs on the main thread.
    unsafe {
        let level = CGShieldingWindowLevel() as isize + 1;
        let _: () = msg_send![ns_window, setLevel: level];
    }
    Ok(())
}
