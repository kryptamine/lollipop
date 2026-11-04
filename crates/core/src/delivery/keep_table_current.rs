use super::manifest::{Manifest, MANIFEST_OBJECT};
use super::manifest_key::ManifestKey;
use super::table_channel::TableChannel;
use super::table_file::TableFile;
use crate::benchmarks::BenchmarkTable;

/// Never returns. Checks now and at each request, and only stages a table: the poll loop
/// swaps it in, as only it knows no game is running. A failed check waits for the next.
pub async fn keep_table_current(table: &TableFile, channel: &TableChannel, key: &ManifestKey) {
    loop {
        if check(table, channel, key).await.is_none() {
            log::warn!("Benchmark table check failed");
        }
        table.check_requested().await;
    }
}

async fn check(table: &TableFile, channel: &TableChannel, key: &ManifestKey) -> Option<()> {
    let manifest = channel.download(MANIFEST_OBJECT).await?;
    let manifest = Manifest::parse(&manifest, key)?;
    if !table.needs_download(&manifest) {
        return Some(());
    }
    let body = channel.download(&manifest.table_object()).await?;
    if !manifest.names(&body) || BenchmarkTable::parse(&body).is_none() {
        return None;
    }
    table.stage(&manifest, &body).ok()
}
