use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lollipop_core::settings::Settings;
use tauri::{AppHandle, Emitter, Manager};

/// The settings of this run, shared by the settings window and the poll loop.
#[derive(Clone)]
pub(crate) struct SettingsStore {
    app: AppHandle,
    path: PathBuf,
    settings: Arc<Mutex<Settings>>,
}

impl SettingsStore {
    pub(crate) fn open(app: &AppHandle) -> tauri::Result<Self> {
        let path = app.path().app_config_dir()?.join("settings.json");
        let settings = Arc::new(Mutex::new(Settings::load(&path)));
        Ok(Self {
            app: app.clone(),
            path,
            settings,
        })
    }

    pub(crate) fn read<T>(&self, read: impl FnOnce(&Settings) -> T) -> T {
        read(&self.lock())
    }

    /// A failed save still applies for this run: a full disk should not undo a click.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "the lock must span the emit, see emit_switches"
    )]
    pub(crate) fn change(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = self.lock();
        change(&mut settings);
        let _ = settings.save(&self.path);
        self.emit_switches(&settings);
    }

    /// The view can load after any one event, so it is told again on every tick.
    pub(crate) fn publish_switches(&self) {
        self.emit_switches(&self.lock());
    }

    // Called with the lock held, so a tick cannot overwrite a newer click with an older value.
    fn emit_switches(&self, settings: &Settings) {
        let _ = self.app.emit("widget-switches", settings.widget_switches());
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.settings.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
