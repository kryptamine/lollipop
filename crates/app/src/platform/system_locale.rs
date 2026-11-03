use std::ffi::{c_char, CStr};

use objc2::{class, msg_send, runtime::AnyObject};

/// The first language in System Settings' preferred list, such as `ru-RU`.
pub(crate) fn system_locale() -> Option<String> {
    // SAFETY: `preferredLanguages` always answers an NSArray of NSStrings, maybe empty,
    // and `firstObject` answers nil for an empty one.
    unsafe {
        let languages: *mut AnyObject = msg_send![class!(NSLocale), preferredLanguages];
        let first: *mut AnyObject = msg_send![languages, firstObject];
        if first.is_null() {
            return None;
        }
        let text: *const c_char = msg_send![first, UTF8String];
        Some(CStr::from_ptr(text).to_string_lossy().into_owned())
    }
}
