//! The player's OpenAI key as a generic password in the login Keychain: see ADR 0016.
//! A write or delete can fail on a denied access prompt, so callers read back the result.

use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

const SERVICE: &str = "dev.lollipop.overlay";
const ACCOUNT: &str = "openai-api-key";

/// A missing item, a denied prompt or bytes that are not text all read as no key.
pub(crate) fn read() -> Option<String> {
    let bytes = get_generic_password(SERVICE, ACCOUNT).ok()?;
    String::from_utf8(bytes).ok()
}

pub(crate) fn write(key: &str) {
    let _ = set_generic_password(SERVICE, ACCOUNT, key.as_bytes());
}

pub(crate) fn delete() {
    let _ = delete_generic_password(SERVICE, ACCOUNT);
}
