//! The settings file on a real disk, the only boundary settings have.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

use std::fs;
use std::path::PathBuf;

use lollipop_core::advice::{AdviceCall, Language, Trigger, TRIGGERS};
use lollipop_core::settings::{AdviceLanguage, Settings, ADVICE_WIDGET, WIDGETS};
use serde_json::json;
use tempfile::TempDir;

fn settings_file() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    (dir, path)
}

fn all_widgets_off() -> Settings {
    let mut settings = Settings::default();
    for widget in &WIDGETS {
        settings.set_widget(widget.id, false);
    }
    settings
}

#[test]
fn a_first_launch_has_every_widget_on_and_launch_at_login_off() {
    let (_dir, path) = settings_file();
    let settings = Settings::load(&path);
    assert!(WIDGETS.iter().all(|widget| settings.widget_on(widget.id)));
    assert!(!settings.launch_at_login);
}

#[test]
fn switches_survive_a_restart() {
    let (_dir, path) = settings_file();
    let mut settings = all_widgets_off();
    settings.launch_at_login = true;
    settings.save(&path).unwrap();

    let restarted = Settings::load(&path);
    assert!(WIDGETS.iter().all(|widget| !restarted.widget_on(widget.id)));
    assert!(restarted.launch_at_login);
}

// Switching back has to win over the value already on disk.
#[test]
fn a_later_save_replaces_the_earlier_one() {
    let (_dir, path) = settings_file();
    let mut settings = all_widgets_off();
    settings.save(&path).unwrap();
    settings.set_widget(WIDGETS[0].id, true);
    settings.save(&path).unwrap();

    assert!(Settings::load(&path).widget_on(WIDGETS[0].id));
}

// The app's config directory does not exist until the first switch is flipped.
#[test]
fn the_first_save_creates_the_directory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not").join("yet").join("settings.json");
    Settings::default().save(&path).unwrap();
    assert!(path.exists());
}

// A file written before a Widget existed must not read as that Widget being off.
#[test]
fn a_widget_added_later_starts_on() {
    let (_dir, path) = settings_file();
    fs::write(&path, r#"{ "widgets": { "markerStrip": false } }"#).unwrap();

    let settings = Settings::load(&path);
    assert!(!settings.widget_on("markerStrip"));
    assert!(settings.widget_on("aWidgetFromALaterVersion"));
    assert!(!settings.launch_at_login);
}

// Going back to an older build must not throw away what the player chose.
#[test]
fn fields_this_version_does_not_know_are_ignored() {
    let (_dir, path) = settings_file();
    fs::write(&path, r#"{ "launchAtLogin": true, "theme": "pink" }"#).unwrap();
    assert!(Settings::load(&path).launch_at_login);
}

#[test]
fn a_damaged_file_gives_the_defaults() {
    let (_dir, path) = settings_file();
    fs::write(&path, "{ not json").unwrap();

    let settings = Settings::load(&path);
    assert!(settings.widget_on("markerStrip"));
    assert!(!settings.launch_at_login);
}

// The view reads this shape: see WidgetSwitches in ui/src/bridge.ts.
#[test]
fn the_view_is_told_about_every_widget() {
    assert_eq!(
        serde_json::to_value(Settings::default().widget_switches()).unwrap(),
        json!({ "adviceWidget": true, "markerStrip": true, "timingHints": true })
    );
    assert_eq!(
        serde_json::to_value(all_widgets_off().widget_switches()).unwrap(),
        json!({ "adviceWidget": false, "markerStrip": false, "timingHints": false })
    );
}

// The settings window reads this shape: see Switches in ui/src/bridge.ts.
#[test]
fn the_settings_window_is_told_every_switch() {
    let mut settings = Settings::default();
    settings.launch_at_login = true;
    settings.set_widget("timingHints", false);
    assert_eq!(
        serde_json::to_value(settings.switches()).unwrap(),
        json!({
            "launchAtLogin": true,
            "widgets": { "adviceWidget": true, "markerStrip": true, "timingHints": false },
            "triggers": {
                "backToBase": true,
                "death": true,
                "loadingScreen": true,
                "turningPoint": true,
            },
            "adviceLanguage": "system",
            "voiceover": false,
        })
    );
}

// The window's ids arrive over IPC, and a stray one must not land in the file.
#[test]
fn a_switch_for_no_widget_is_not_stored() {
    let (_dir, path) = settings_file();
    let mut settings = Settings::default();
    settings.set_widget("notAWidget", false);
    settings.save(&path).unwrap();

    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["widgets"], json!({}));
}

const KEY: &str = "sk-proj-Fake0123abcd";

fn call_with(settings: &Settings, locale: Option<&str>) -> Option<AdviceCall> {
    settings.advice_call(Trigger::LoadingScreen, Some(KEY.to_string()), locale)
}

#[test]
fn a_first_launch_has_every_trigger_on_and_the_system_language() {
    let (_dir, path) = settings_file();
    let settings = Settings::load(&path);
    assert!(TRIGGERS.iter().all(|&trigger| settings.trigger_on(trigger)));
    assert_eq!(settings.advice_language, AdviceLanguage::System);
}

#[test]
fn advice_switches_survive_a_restart() {
    let (_dir, path) = settings_file();
    let mut settings = Settings::default();
    settings.set_trigger("loadingScreen", false);
    settings.advice_language = AdviceLanguage::Russian;
    settings.save(&path).unwrap();

    let restarted = Settings::load(&path);
    assert!(!restarted.trigger_on(Trigger::LoadingScreen));
    assert_eq!(restarted.advice_language, AdviceLanguage::Russian);
}

#[test]
fn a_switch_for_no_trigger_is_not_stored() {
    let (_dir, path) = settings_file();
    let mut settings = Settings::default();
    settings.set_trigger("notATrigger", false);
    settings.save(&path).unwrap();

    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["triggers"], json!({}));
}

// The Advice Widget's switch is the only Advice on/off: off means no OpenAI call at all.
#[test]
fn openai_is_called_only_with_a_key_the_widget_and_the_trigger_on() {
    let mut settings = Settings::default();
    assert!(call_with(&settings, None).is_some());
    assert!(settings
        .advice_call(Trigger::LoadingScreen, None, None)
        .is_none());

    settings.set_widget(ADVICE_WIDGET, false);
    assert!(call_with(&settings, None).is_none());

    settings.set_widget(ADVICE_WIDGET, true);
    settings.set_trigger(Trigger::LoadingScreen.id(), false);
    assert!(call_with(&settings, None).is_none());
}

#[test]
fn the_call_carries_the_key_it_was_given() {
    let call = call_with(&Settings::default(), None).unwrap();
    assert_eq!(call.key, KEY);
}

// Advice is written in Russian or English only, so any other system language reads English.
#[test]
fn the_system_default_follows_the_system_language() {
    let settings = Settings::default();
    let language = |locale| call_with(&settings, locale).unwrap().language;
    assert_eq!(language(Some("ru-RU")), Language::Russian);
    assert_eq!(language(Some("ru")), Language::Russian);
    assert_eq!(language(Some("en-GB")), Language::English);
    assert_eq!(language(Some("de-DE")), Language::English);
    assert_eq!(language(None), Language::English);
}

#[test]
fn a_chosen_language_wins_over_the_system_one() {
    let mut settings = Settings::default();
    settings.advice_language = AdviceLanguage::English;
    let call = call_with(&settings, Some("ru-RU")).unwrap();
    assert_eq!(call.language, Language::English);

    settings.advice_language = AdviceLanguage::Russian;
    let call = call_with(&settings, Some("en-US")).unwrap();
    assert_eq!(call.language, Language::Russian);
}

// The view sends the language as these strings: see AdviceLanguage in ui/src/bridge.ts.
#[test]
fn the_language_is_kept_as_the_view_names_it() {
    let (_dir, path) = settings_file();
    fs::write(&path, r#"{ "adviceLanguage": "ru" }"#).unwrap();
    assert_eq!(
        Settings::load(&path).advice_language,
        AdviceLanguage::Russian
    );
}

#[test]
fn voiceover_starts_off_and_survives_a_restart() {
    let (_dir, path) = settings_file();
    let mut settings = Settings::load(&path);
    assert!(!settings.voiceover);
    assert!(!settings.reads_aloud());

    settings.voiceover = true;
    settings.save(&path).unwrap();
    assert!(Settings::load(&path).reads_aloud());
}

// Voiceover reads only what the Widget shows, so the Widget off silences it.
#[test]
fn voiceover_is_silent_with_the_advice_widget_off() {
    let mut settings = Settings::default();
    settings.voiceover = true;
    settings.set_widget(ADVICE_WIDGET, false);
    assert!(!settings.reads_aloud());
}

// The key reaches OpenAI only, so no log line may print it.
#[test]
fn a_call_never_prints_its_key() {
    let call = call_with(&Settings::default(), None).unwrap();
    assert!(!format!("{call:?}").contains(KEY));
}
