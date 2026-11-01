use std::sync::Arc;

use lollipop_core::delivery::{
    keep_table_current, ManifestKey, TableChannel, TableFile, TABLE_CHANNEL_URL,
};

pub(crate) async fn run(table: Arc<TableFile>) {
    let channel = TableChannel::new(TABLE_CHANNEL_URL).ok();
    if let (Some(channel), Some(key)) = (channel, ManifestKey::compiled_in()) {
        keep_table_current(&table, &channel, &key).await;
    }
}
