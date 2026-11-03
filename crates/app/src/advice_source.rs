use lollipop_core::advice::Advisor;

use crate::openai_key::OpenAiKey;
use crate::settings_store::SettingsStore;

/// Advice as the settings and the saved key allow it at the moment each Trigger fires.
pub(crate) fn advisor(store: SettingsStore, key: OpenAiKey) -> Advisor {
    #[cfg(target_os = "macos")]
    let locale = crate::platform::system_locale();
    #[cfg(not(target_os = "macos"))]
    let locale: Option<String> = None;
    let base_url = key.base_url().to_string();
    Advisor::new(&base_url, move |trigger| {
        let key = key.current();
        store.read(|settings| settings.advice_call(trigger, key, locale.as_deref()))
    })
}
