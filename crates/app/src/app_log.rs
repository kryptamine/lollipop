use tauri::{plugin::TauriPlugin, Wry};
use tauri_plugin_log::{log::LevelFilter, RotationStrategy, TimezoneStrategy};

// The default 40 KB cap would rotate away a game's lines before it ended.
const MAX_FILE_BYTES: u128 = 5_000_000;

/// Writes to `~/Library/Logs/<identifier>/` and standard output, Lollipop's own lines
/// at debug and its dependencies' from warnings up.
pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_log::Builder::new()
        .level(LevelFilter::Warn)
        .level_for("lollipop", LevelFilter::Debug)
        .level_for("lollipop_core", LevelFilter::Debug)
        .max_file_size(MAX_FILE_BYTES)
        .rotation_strategy(RotationStrategy::KeepSome(3))
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .build()
}
