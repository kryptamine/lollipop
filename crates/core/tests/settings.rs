//! The settings file on a real disk, the only boundary settings have.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

use std::fs;
use std::path::PathBuf;

use lollipop_core::settings::{Settings, WIDGETS};
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
        json!({ "markerStrip": true, "timingHints": true })
    );
    assert_eq!(
        serde_json::to_value(all_widgets_off().widget_switches()).unwrap(),
        json!({ "markerStrip": false, "timingHints": false })
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
            "widgets": { "markerStrip": true, "timingHints": false },
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
