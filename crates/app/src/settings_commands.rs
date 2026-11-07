//! What the settings window may ask of the app. Each change answers with every
//! switch, so the window draws what was kept rather than what was clicked.
#![allow(
    clippy::needless_pass_by_value,
    reason = "a Tauri command receives its arguments by value"
)]

use lollipop_core::settings::{AdviceLanguage, Settings, Switches};
use tauri::{AppHandle, State};

use crate::launch_at_login;
use crate::settings_store::SettingsStore;
use crate::voiceover::Voiceover;

#[tauri::command]
pub(crate) fn read_switches(store: State<'_, SettingsStore>) -> Switches {
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_widget(store: State<'_, SettingsStore>, id: &str, is_on: bool) -> Switches {
    store.change(|settings| settings.set_widget(id, is_on));
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_trigger(store: State<'_, SettingsStore>, id: &str, is_on: bool) -> Switches {
    store.change(|settings| settings.set_trigger(id, is_on));
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_advice_language(
    store: State<'_, SettingsStore>,
    language: AdviceLanguage,
) -> Switches {
    store.change(|settings| settings.advice_language = language);
    store.read(Settings::switches)
}

/// A login item the OS refused keeps the old value, which the window then shows.
#[tauri::command]
pub(crate) fn set_launch_at_login(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    is_on: bool,
) -> Switches {
    if launch_at_login::apply(&app, is_on).is_ok() {
        store.change(|settings| settings.launch_at_login = is_on);
    }
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_voiceover(store: State<'_, SettingsStore>, is_on: bool) -> Switches {
    store.change(|settings| settings.voiceover = is_on);
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_recap_window(store: State<'_, SettingsStore>, is_on: bool) -> Switches {
    store.change(|settings| settings.recap.window = is_on);
    store.read(Settings::switches)
}

#[tauri::command]
pub(crate) fn set_lessons(store: State<'_, SettingsStore>, is_on: bool) -> Switches {
    store.change(|settings| settings.recap.lessons = is_on);
    store.read(Settings::switches)
}

/// For the Advice language as kept now; the window asks again when it changes.
#[tauri::command]
pub(crate) fn read_voice_installed(voiceover: State<'_, Voiceover>) -> bool {
    voiceover.voice_installed()
}
