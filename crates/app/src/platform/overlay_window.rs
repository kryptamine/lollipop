use objc2::{msg_send, runtime::AnyObject};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGShieldingWindowLevel() -> i32;
}

/// League's Fullscreen windows sit at the shield level, so the overlay goes one above it.
/// Ordered in once and never made key: from here on only alpha changes, see ADR 0006.
pub(crate) fn order_in_hidden(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let ns_window = window.ns_window()?.cast::<AnyObject>();
    // SAFETY: `ns_window` is the live NSWindow Tauri owns, and setup runs on the main thread.
    unsafe {
        #[allow(clippy::as_conversions, reason = "i32 always fits in isize on macOS")]
        let level = CGShieldingWindowLevel() as isize + 1;
        let _: () = msg_send![ns_window, setLevel: level];
        let _: () = msg_send![ns_window, setAlphaValue: 0.0_f64];
        let _: () = msg_send![ns_window, orderFrontRegardless];
    }
    Ok(())
}

/// Must run on the main thread.
pub(crate) fn set_shown(window: &tauri::WebviewWindow, shown: bool) -> tauri::Result<()> {
    let ns_window = window.ns_window()?.cast::<AnyObject>();
    let alpha = if shown { 1.0_f64 } else { 0.0_f64 };
    // SAFETY: `ns_window` is the live NSWindow Tauri owns, on the main thread by contract.
    unsafe {
        let _: () = msg_send![ns_window, setAlphaValue: alpha];
    }
    Ok(())
}
