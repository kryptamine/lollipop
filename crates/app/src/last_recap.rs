use std::sync::{Arc, Mutex, PoisonError};

use lollipop_core::live::{Recap, RecapNews};
use tauri::{AppHandle, Emitter, Manager};

use crate::settings_store::SettingsStore;

/// The last game's Recap, kept for a Recap window that loads after the event that
/// sent it.
#[derive(Clone)]
pub(crate) struct LastRecap {
    app: AppHandle,
    recap: Arc<Mutex<Option<Recap>>>,
}

impl LastRecap {
    pub(crate) fn new(app: AppHandle) -> Self {
        Self {
            app,
            recap: Arc::default(),
        }
    }

    /// Sends a new Recap to the window, and opens the window when told to, unless the
    /// player switched that off.
    pub(crate) fn show(&self, RecapNews { recap, open }: RecapNews) {
        if let Some(recap) = recap {
            let _ = self.app.emit("recap", &recap);
            *self.recap.lock().unwrap_or_else(PoisonError::into_inner) = Some(recap);
        }
        let switched_on = || {
            self.app
                .state::<SettingsStore>()
                .read(|settings| settings.recap.window)
        };
        if open && switched_on() {
            let _ = crate::recap_window::open(&self.app);
        }
    }

    pub(crate) fn get(&self) -> Option<Recap> {
        self.recap
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}
