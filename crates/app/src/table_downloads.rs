use std::sync::Arc;

use lollipop_core::delivery::{
    keep_table_current, ReleaseChannel, ReleaseToken, TableFile, GITHUB_API, RELEASE_REPO,
};

/// With no token the app simply has no downloads: whatever table is on disk stays.
pub(crate) async fn run(table: Arc<TableFile>) {
    // `security` can wait on a locked Keychain, so it stays off the async threads.
    let token = tauri::async_runtime::spawn_blocking(ReleaseToken::from_keychain).await;
    let channel = token
        .ok()
        .flatten()
        .and_then(|token| ReleaseChannel::new(GITHUB_API, RELEASE_REPO, token).ok());
    if let Some(channel) = channel {
        keep_table_current(&table, &channel).await;
    }
}
