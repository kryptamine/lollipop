use lollipop_core::advice::Advisor;
use tauri::{AppHandle, Emitter};

use crate::openai_key::OpenAiKey;
use crate::settings_store::SettingsStore;

/// Advice as the settings and the saved key allow it at the moment each Trigger fires.
pub(crate) fn advisor(app: AppHandle, store: SettingsStore, key: OpenAiKey) -> Advisor {
    let locale = crate::platform::system_locale();
    let base_url = key.base_url().to_string();
    let refused = key.clone();
    let (language_store, language_locale) = (store.clone(), locale.clone());
    let (lessons_store, lessons_locale, lessons_key) = (store.clone(), locale.clone(), key.clone());
    Advisor::new(&base_url, move |trigger| {
        let key = key.current();
        store.read(|settings| settings.advice_call(trigger, key, locale.as_deref()))
    })
    .reviewing(move || {
        let key = lessons_key.current();
        lessons_store.read(|settings| settings.lessons_call(key, lessons_locale.as_deref()))
    })
    .in_language(move || {
        language_store.read(|settings| {
            settings
                .advice_language
                .language(language_locale.as_deref())
        })
    })
    .logging_to(log)
    .on_key_failure(move |failure| {
        refused.fail(failure);
        let _ = app.emit("openai-key-failure", failure);
    })
}

#[allow(
    clippy::print_stderr,
    reason = "the app has no log but its standard error"
)]
fn log(line: &str) {
    eprintln!("{line}");
}
