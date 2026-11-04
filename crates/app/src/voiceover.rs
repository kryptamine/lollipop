use std::sync::{Arc, Mutex, PoisonError};

use lollipop_core::live::ShownAdvice;
use lollipop_core::settings::Settings;
use lollipop_core::voiceover::{Reader, Speech};
use tauri::WebviewWindow;

use crate::settings_store::SettingsStore;

// The voice loads cold in 239-620 ms; a silent line at game start hides that from the
// first Advice.
const WARM_UP_LINE: &str = "Lollipop";

/// Reads the Advice and the Build change the Widgets show, as the settings allow. Its state changes only
/// on the main thread, where the synthesizer lives, so no tick overtakes a hide.
#[derive(Clone)]
pub(crate) struct Voiceover {
    window: WebviewWindow,
    store: SettingsStore,
    locale: Option<String>,
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    reader: Reader,
    advice: Option<ShownAdvice>,
    build_change: Option<String>,
    overlay_shown: bool,
}

impl Voiceover {
    pub(crate) fn new(window: WebviewWindow, store: SettingsStore) -> Self {
        #[cfg(target_os = "macos")]
        let locale = crate::platform::system_locale();
        #[cfg(not(target_os = "macos"))]
        let locale: Option<String> = None;
        Self {
            window,
            store,
            locale,
            state: Arc::default(),
        }
    }

    /// Whether a voice for the Advice language is installed; without one Voiceover is silent.
    pub(crate) fn voice_installed(&self) -> bool {
        #[cfg(target_os = "macos")]
        return crate::platform::best_voice(self.voice_tag()).is_some();
        #[cfg(not(target_os = "macos"))]
        false
    }

    pub(crate) fn warm_up(&self) {
        if self.store.read(Settings::reads_aloud) {
            let this = self.clone();
            let _ = self.window.run_on_main_thread(move || {
                this.speak(WARM_UP_LINE, 0.0);
            });
        }
    }

    pub(crate) fn show(&self, advice: Option<ShownAdvice>, build_change: Option<String>) {
        self.update(move |state| {
            state.advice = advice;
            state.build_change = build_change;
        });
    }

    pub(crate) fn set_overlay_shown(&self, shown: bool) {
        self.update(move |state| state.overlay_shown = shown);
    }

    fn update(&self, change: impl FnOnce(&mut State) + Send + 'static) {
        let this = self.clone();
        let _ = self.window.run_on_main_thread(move || {
            let reads_aloud = this.store.read(Settings::reads_aloud);
            let reads_changes = this.store.read(Settings::reads_build_changes);
            let speech = {
                let mut state = this.state.lock().unwrap_or_else(PoisonError::into_inner);
                change(&mut state);
                let shown = state.overlay_shown;
                let heard = state.advice.clone().filter(|_| reads_aloud && shown);
                let heard_change = state
                    .build_change
                    .clone()
                    .filter(|_| reads_changes && shown);
                state.reader.follow(heard.as_ref(), heard_change.as_deref())
            };
            match speech {
                Some(Speech::Read(text)) => this.speak(&text, 1.0),
                Some(Speech::Stop) => this.stop(),
                None => {}
            }
        });
    }

    fn voice_tag(&self) -> &'static str {
        self.store.read(|settings| {
            settings
                .advice_language
                .language(self.locale.as_deref())
                .voice_tag()
        })
    }

    // Must run on the main thread.
    #[allow(
        clippy::unused_self,
        reason = "the speaking half of the struct exists on macOS only"
    )]
    fn speak(&self, text: &str, volume: f32) {
        #[cfg(target_os = "macos")]
        if let (Some(main), Some(voice)) = (
            objc2::MainThreadMarker::new(),
            crate::platform::best_voice(self.voice_tag()),
        ) {
            crate::platform::speak(main, text, &voice, volume);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (text, volume);
    }

    // Must run on the main thread.
    #[allow(
        clippy::unused_self,
        reason = "the speaking half of the struct exists on macOS only"
    )]
    fn stop(&self) {
        #[cfg(target_os = "macos")]
        if let Some(main) = objc2::MainThreadMarker::new() {
            crate::platform::stop_speaking(main);
        }
    }
}
