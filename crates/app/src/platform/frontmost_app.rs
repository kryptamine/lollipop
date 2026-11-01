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
    // SAFETY: a process-wide singleton, always available in an AppKit app.
    let workspace: *mut AnyObject = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    // SAFETY: `workspace` is the live singleton above.
    let current: *mut AnyObject = unsafe { msg_send![workspace, frontmostApplication] };
    on_change(bundle_id(current));

    let block = RcBlock::new(move |notification: *mut AnyObject| {
        on_change(bundle_id(activated_app(notification)));
    });
    // SAFETY: `workspace` is the live singleton above.
    let center: *mut AnyObject = unsafe { msg_send![workspace, notificationCenter] };
    // SAFETY: a constant AppKit exports, and reading it copies a pointer.
    let name = unsafe { NSWorkspaceDidActivateApplicationNotification };
    let nil: *mut AnyObject = std::ptr::null_mut();
    // SAFETY: `center` is live, the block outlives the call because the center copies it,
    // and a nil queue runs it on the posting thread, which is main for workspace events.
    let _observer: *mut AnyObject = unsafe {
        msg_send![center, addObserverForName: name, object: nil, queue: nil, usingBlock: &*block]
    };
}

fn activated_app(notification: *mut AnyObject) -> *mut AnyObject {
    // SAFETY: `notification` is the live NSNotification AppKit passes to the observer.
    let info: *mut AnyObject = unsafe { msg_send![notification, userInfo] };
    // SAFETY: a constant AppKit exports, and reading it copies a pointer.
    let key = unsafe { NSWorkspaceApplicationKey };
    // SAFETY: this notification's `userInfo` is a live NSDictionary, and `objectForKey:`
    // answers nil for a missing key.
    unsafe { msg_send![info, objectForKey: key] }
}

// `app` comes from AppKit: null, or a live NSRunningApplication.
fn bundle_id(app: *mut AnyObject) -> Option<String> {
    if app.is_null() {
        return None;
    }
    // SAFETY: `app` is live and not null, checked above.
    let id: *mut AnyObject = unsafe { msg_send![app, bundleIdentifier] };
    if id.is_null() {
        return None;
    }
    // SAFETY: `id` is a live NSString, not null, checked above.
    let text: *const c_char = unsafe { msg_send![id, UTF8String] };
    // SAFETY: `UTF8String` returns a NUL-terminated buffer that lives as long as `id`,
    // and it is copied out before this function returns.
    let bundle = unsafe { CStr::from_ptr(text) };
    Some(bundle.to_string_lossy().into_owned())
}
