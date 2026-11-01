use std::ffi::{c_char, CStr};

use block2::RcBlock;
use objc2::{class, msg_send, runtime::AnyObject};

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    static NSWorkspaceDidActivateApplicationNotification: *mut AnyObject;
    static NSWorkspaceApplicationKey: *mut AnyObject;
}

/// Reports the frontmost app's bundle id now and on every activation, for the life of
/// the process. Launching takes no focus, so no notification says the game is already
/// in front. Must run on the main thread, where the callback then runs too.
pub(crate) fn watch_frontmost(on_change: impl Fn(Option<String>) + 'static) {
    // SAFETY: NSWorkspace and its notification center are process-wide singletons, and
    // the notification's `userInfo` always holds an NSRunningApplication under this key.
    unsafe {
        let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        let current: *mut AnyObject = msg_send![workspace, frontmostApplication];
        on_change(bundle_id(current));

        let block = RcBlock::new(move |notification: *mut AnyObject| {
            let info: *mut AnyObject = msg_send![notification, userInfo];
            let app: *mut AnyObject = msg_send![info, objectForKey: NSWorkspaceApplicationKey];
            on_change(bundle_id(app));
        });
        let center: *mut AnyObject = msg_send![workspace, notificationCenter];
        let nil: *mut AnyObject = std::ptr::null_mut();
        let _observer: *mut AnyObject = msg_send![
            center,
            addObserverForName: NSWorkspaceDidActivateApplicationNotification,
            object: nil,
            queue: nil,
            usingBlock: &*block
        ];
    }
}

/// # Safety
/// `app` must be null or a live `NSRunningApplication`.
unsafe fn bundle_id(app: *mut AnyObject) -> Option<String> {
    if app.is_null() {
        return None;
    }
    let id: *mut AnyObject = msg_send![app, bundleIdentifier];
    if id.is_null() {
        return None;
    }
    let text: *const c_char = msg_send![id, UTF8String];
    Some(CStr::from_ptr(text).to_string_lossy().into_owned())
}
