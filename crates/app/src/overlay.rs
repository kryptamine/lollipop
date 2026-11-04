use std::sync::{Arc, Mutex, PoisonError};

use lollipop_core::overlay::Visibility;
use tauri::WebviewWindow;

use crate::voiceover::Voiceover;

/// Applies `Visibility` to the window. Both inputs run on the main thread, so a
/// slower poll can never overwrite a newer frontmost change.
#[derive(Clone)]
pub(crate) struct Overlay {
    window: WebviewWindow,
    visibility: Arc<Mutex<Visibility>>,
    voiceover: Voiceover,
}

impl Overlay {
    pub(crate) fn new(window: WebviewWindow, game_bundle_id: &str, voiceover: Voiceover) -> Self {
        let visibility = Arc::new(Mutex::new(Visibility::new(game_bundle_id)));
        Self {
            window,
            visibility,
            voiceover,
        }
    }

    pub(crate) fn set_frontmost(&self, bundle_id: Option<String>) {
        log::debug!("frontmost app: {bundle_id:?}");
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
            let (was, was_shown) = (format!("{visibility:?}"), visibility.shown());
            change(&mut visibility);
            if format!("{visibility:?}") != was {
                log::debug!("overlay {visibility:?}, shown: {}", visibility.shown());
            }
            if visibility.shown() != was_shown {
                this.voiceover.set_overlay_shown(visibility.shown());
                if let Err(error) = crate::platform::set_shown(&this.window, visibility.shown()) {
                    log::warn!("overlay not shown or hidden: {error}");
                }
            }
        });
    }
}
