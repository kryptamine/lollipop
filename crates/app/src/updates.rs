use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use lollipop_core::updates::{UpdateSchedule, UpdateStep};
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};

type Downloaded = Option<(Update, Vec<u8>)>;

/// Runs the steps of `UpdateSchedule` on the updater plugin. A failed step is
/// dropped: the next game's end checks again.
#[derive(Clone)]
pub(crate) struct Updates {
    app: AppHandle,
    schedule: Arc<Mutex<UpdateSchedule>>,
    downloaded: Arc<Mutex<Downloaded>>,
}

impl Updates {
    pub(crate) fn new(app: AppHandle) -> Self {
        Self {
            app,
            schedule: Arc::default(),
            downloaded: Arc::default(),
        }
    }

    pub(crate) fn set_between_games(&self, between_games: bool) {
        let step = self.schedule().set_between_games(between_games);
        self.take(step);
    }

    fn take(&self, step: Option<UpdateStep>) {
        match step {
            Some(UpdateStep::Check) => {
                tauri::async_runtime::spawn(self.clone().check_and_download());
            }
            Some(UpdateStep::Install) => {
                let this = self.clone();
                tauri::async_runtime::spawn_blocking(move || this.install());
            }
            None => {}
        }
    }

    async fn check_and_download(self) {
        let Ok(updater) = self.app.updater() else {
            return;
        };
        let Ok(Some(update)) = updater.check().await else {
            return;
        };
        let Ok(bytes) = update.download(|_, _| {}, || {}).await else {
            return;
        };
        *lock(&self.downloaded) = Some((update, bytes));
        let step = self.schedule().update_downloaded();
        self.take(step);
    }

    fn install(&self) {
        let Some((update, bytes)) = lock(&self.downloaded).take() else {
            return;
        };
        if update.install(bytes).is_ok() {
            self.app.restart();
        }
    }

    fn schedule(&self) -> MutexGuard<'_, UpdateSchedule> {
        lock(&self.schedule)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
