use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lollipop_core::advice::{check_key, masked_key, KeyCheck, OPENAI_URL};

use crate::key_save::KeySave;
#[cfg(target_os = "macos")]
use crate::keychain;

#[derive(Clone, Copy)]
enum Keeper {
    #[cfg(target_os = "macos")]
    Keychain,
    /// A `make dev-mock` key, which must not overwrite the player's real one.
    ThisRun,
}

/// The player's OpenAI key for this run. The full key never leaves the app: the view
/// only ever gets [`masked_key`]. No `Debug`, so no log line can print it.
#[derive(Clone)]
pub(crate) struct OpenAiKey {
    base_url: String,
    keeper: Keeper,
    key: Arc<Mutex<Option<String>>>,
}

impl OpenAiKey {
    pub(crate) fn open() -> Self {
        let base_url = crate::dev_setting("LOLLIPOP_OPENAI_URL", OPENAI_URL);
        let dev_key = std::env::var("LOLLIPOP_OPENAI_KEY")
            .ok()
            .filter(|_| cfg!(debug_assertions));
        let (keeper, key) = dev_key.map_or_else(stored, |key| (Keeper::ThisRun, Some(key)));
        Self {
            base_url,
            keeper,
            key: Arc::new(Mutex::new(key)),
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The full key, which only an Advice call may be given.
    pub(crate) fn current(&self) -> Option<String> {
        self.lock().clone()
    }

    pub(crate) fn masked(&self) -> Option<String> {
        self.lock().as_deref().map(masked_key)
    }

    /// Only a key OpenAI answered 200 for is kept; any other answer keeps the old one.
    pub(crate) async fn save(&self, key: &str) -> KeySave {
        match check_key(&self.base_url, key).await {
            KeyCheck::Accepted => {}
            KeyCheck::Rejected => return KeySave::Rejected,
            KeyCheck::Unreachable => return KeySave::Unreachable,
        }
        let mut kept = self.lock();
        *kept = match self.keeper {
            #[cfg(target_os = "macos")]
            Keeper::Keychain => {
                keychain::write(key);
                keychain::read()
            }
            Keeper::ThisRun => Some(key.to_string()),
        };
        if kept.as_deref() == Some(key) {
            KeySave::Saved {
                masked: masked_key(key),
            }
        } else {
            KeySave::NotKept
        }
    }

    /// Answers the key still kept, masked: a Keychain that refused the delete keeps it.
    pub(crate) fn remove(&self) -> Option<String> {
        let mut kept = self.lock();
        *kept = match self.keeper {
            #[cfg(target_os = "macos")]
            Keeper::Keychain => {
                keychain::delete();
                keychain::read()
            }
            Keeper::ThisRun => None,
        };
        kept.as_deref().map(masked_key)
    }

    fn lock(&self) -> MutexGuard<'_, Option<String>> {
        self.key.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(target_os = "macos")]
fn stored() -> (Keeper, Option<String>) {
    (Keeper::Keychain, keychain::read())
}

#[cfg(not(target_os = "macos"))]
const fn stored() -> (Keeper, Option<String>) {
    (Keeper::ThisRun, None)
}
