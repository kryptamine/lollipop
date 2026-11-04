use std::path::Path;

use super::connection::Connection;
use super::lockfile::Lockfile;
use super::patch::Patch;

/// The patch on the loading screen, before `GameInputs` are read.
pub(crate) async fn loading_patch(http: &reqwest::Client, lockfile: &Path) -> Option<Patch> {
    let lockfile = Lockfile::read(lockfile)?;
    Connection::new(http, lockfile)
        .get::<String>("/lol-patch/v1/game-version")
        .await
        .as_deref()
        .and_then(Patch::parse)
}
