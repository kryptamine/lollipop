use objc2::{msg_send, runtime::AnyObject};

/// `orderFront:` leaves a window of an inactive app behind the active one, and Lollipop
/// never activates on its own, so the window is ordered in regardless.
pub(crate) fn show_without_focus(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let shown = window.clone();
    window.run_on_main_thread(move || {
        let Ok(ns_window) = shown.ns_window() else {
            return;
        };
        let ns_window = ns_window.cast::<AnyObject>();
        // SAFETY: `ns_window` is the live NSWindow Tauri owns, used on the main thread.
        unsafe {
            let _: () = msg_send![ns_window, orderFrontRegardless];
        }
    })
}
