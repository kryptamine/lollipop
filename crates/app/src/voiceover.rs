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
        let locale = crate::platform::system_locale();
        Self {
            window,
            store,
            locale,
            state: Arc::default(),
        }
    }

    /// Whether a voice for the Advice language is installed; without one Voiceover is silent.
    pub(crate) fn voice_installed(&self) -> bool {
        crate::platform::voice_installed(self.voice_tag())
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
                Some(Speech::Stop) => crate::platform::stop_speaking(),
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

    fn speak(&self, text: &str, volume: f32) {
        crate::platform::speak(text, self.voice_tag(), volume);
    }
}
