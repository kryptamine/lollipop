//! No keychain is reachable here, so a saved key lasts for this run only.

use std::sync::{Mutex, MutexGuard, PoisonError};

static KEY: Mutex<Option<String>> = Mutex::new(None);

pub(crate) fn read() -> Option<String> {
    key().clone()
}

pub(crate) fn write(key_text: &str) {
    *key() = Some(key_text.to_string());
}

pub(crate) fn delete() {
    *key() = None;
}

fn key() -> MutexGuard<'static, Option<String>> {
    KEY.lock().unwrap_or_else(PoisonError::into_inner)
}
