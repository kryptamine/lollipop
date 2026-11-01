use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tokio::sync::Notify;

use super::manifest::Manifest;

/// The table file the app reads, and a verified download waiting to replace it.
#[derive(Debug)]
pub struct TableFile {
    live: PathBuf,
    // The sha256 of the staged file, so a leftover from another run is never swapped in.
    // The lock spans the file operations, so a check cannot interleave with a swap.
    staged: Mutex<Option<String>>,
    check_requested: Notify,
}

impl TableFile {
    #[must_use]
    pub fn new(live: PathBuf) -> Self {
        Self {
            live,
            staged: Mutex::new(None),
            check_requested: Notify::new(),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.live
    }

    pub(crate) fn request_check(&self) {
        self.check_requested.notify_one();
    }

    pub(super) async fn check_requested(&self) {
        self.check_requested.notified().await;
    }

    /// Comparing content, not versions, is what follows the manifest backwards and replaces
    /// a live file that no longer reads. A staged table it no longer names is dropped.
    pub(super) fn needs_download(&self, manifest: &Manifest) -> bool {
        let mut staged = self.lock();
        let live_is_current = std::fs::read(&self.live).is_ok_and(|live| manifest.names(&live));
        if live_is_current {
            *staged = None;
            return false;
        }
        if staged.as_deref() == Some(manifest.sha256.as_str()) {
            return false;
        }
        *staged = None;
        true
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "a swap must not rename the file while it is being written"
    )]
    pub(super) fn stage(&self, manifest: &Manifest, body: &[u8]) -> std::io::Result<()> {
        let mut staged = self.lock();
        if let Some(directory) = self.live.parent() {
            std::fs::create_dir_all(directory)?;
        }
        let mut file = File::create(self.staged_path())?;
        file.write_all(body)?;
        // On disk before the rename, or a power cut could leave an empty live file.
        file.sync_all()?;
        *staged = Some(manifest.sha256.clone());
        Ok(())
    }

    /// The rename is the atomic step, so a reader sees the old file or the new one, whole.
    /// After a failed one the next check downloads again.
    pub(crate) fn swap_staged(&self) {
        let mut staged = self.lock();
        if staged.take().is_some() {
            let _ = std::fs::rename(self.staged_path(), &self.live);
        }
    }

    // Beside the live file, as a rename is only atomic within one filesystem.
    fn staged_path(&self) -> PathBuf {
        let mut path = self.live.clone().into_os_string();
        path.push(".staged");
        path.into()
    }

    fn lock(&self) -> MutexGuard<'_, Option<String>> {
        self.staged.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
