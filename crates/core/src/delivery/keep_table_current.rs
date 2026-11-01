use super::manifest::{Manifest, MANIFEST_ASSET};
use super::manifest_key::ManifestKey;
use super::release_channel::ReleaseChannel;
use super::table_file::TableFile;
use crate::benchmarks::BenchmarkTable;

/// Never returns. Checks now and at each request, and only stages a table: the poll loop
/// swaps it in, as only it knows no game is running. A failed check waits for the next.
pub async fn keep_table_current(table: &TableFile, channel: &ReleaseChannel, key: &ManifestKey) {
    loop {
        let _ = check(table, channel, key).await;
        table.check_requested().await;
    }
}

async fn check(table: &TableFile, channel: &ReleaseChannel, key: &ManifestKey) -> Option<()> {
    let assets = channel.assets().await?;
    let manifest = channel.download(&assets, MANIFEST_ASSET).await?;
    let manifest = Manifest::parse(&manifest, key)?;
    if !table.needs_download(&manifest) {
        return Some(());
    }
    let body = channel.download(&assets, &manifest.table_asset()).await?;
    if !manifest.names(&body) || BenchmarkTable::parse(&body).is_none() {
        return None;
    }
    table.stage(&manifest, &body).ok()
}
