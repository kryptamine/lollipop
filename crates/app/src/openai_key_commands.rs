//! What the Advice tab may ask about the OpenAI key. None of them answers the full key.
#![allow(
    clippy::needless_pass_by_value,
    reason = "a Tauri command receives its arguments by value"
)]

use std::process::Command;

use lollipop_core::settings::ADVICE_WIDGET;
use tauri::{AppHandle, Manager, State};

use crate::key_save::KeySave;
use crate::openai_key::OpenAiKey;
use crate::settings_store::SettingsStore;

const USAGE_URL: &str = "https://platform.openai.com/usage";

#[tauri::command]
pub(crate) fn read_openai_key(key: State<'_, OpenAiKey>) -> Option<String> {
    key.masked()
}

// Takes the app rather than the state: an async command may not borrow its arguments.
#[tauri::command]
pub(crate) async fn save_openai_key(app: AppHandle, key: String) -> KeySave {
    let openai_key = app.state::<OpenAiKey>().inner().clone();
    openai_key.save(key.trim()).await
}

/// Also turns the Advice Widget off, so a key saved later does not bring Advice back
/// unasked.
#[tauri::command]
pub(crate) fn remove_openai_key(
    key: State<'_, OpenAiKey>,
    store: State<'_, SettingsStore>,
) -> Option<String> {
    store.change(|settings| settings.set_widget(ADVICE_WIDGET, false));
    key.remove()
}

/// Opens a fixed page in the browser, so the view cannot open just any URL.
#[tauri::command]
pub(crate) fn open_openai_usage() {
    let _ = Command::new("/usr/bin/open").arg(USAGE_URL).spawn();
}
