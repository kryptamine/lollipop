use std::sync::{Arc, Mutex, PoisonError};

use lollipop_core::overlay::Visibility;
use tauri::WebviewWindow;

/// Applies `Visibility` to the window. Both inputs run on the main thread, so a
/// slower poll can never overwrite a newer frontmost change.
#[derive(Clone)]
pub(crate) struct Overlay {
    window: WebviewWindow,
    visibility: Arc<Mutex<Visibility>>,
}

impl Overlay {
    pub(crate) fn new(window: WebviewWindow, game_bundle_id: &str) -> Self {
        let visibility = Arc::new(Mutex::new(Visibility::new(game_bundle_id)));
        Self { window, visibility }
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn set_frontmost(&self, bundle_id: Option<String>) {
        self.update(move |visibility| visibility.set_frontmost(bundle_id.as_deref()));
    }

    pub(crate) fn set_match_running(&self, running: bool) {
        self.update(move |visibility| visibility.set_match_running(running));
    }

    fn update(&self, change: impl FnOnce(&mut Visibility) + Send + 'static) {
        let this = self.clone();
        let _ = self.window.run_on_main_thread(move || {
            let mut visibility = this
                .visibility
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let was_shown = visibility.shown();
            change(&mut visibility);
            if visibility.shown() != was_shown {
                #[cfg(target_os = "macos")]
                let _ = crate::platform::set_shown(&this.window, visibility.shown());
            }
        });
    }
}
