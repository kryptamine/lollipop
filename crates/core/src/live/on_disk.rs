use super::game_history::GameHistory;
use crate::delivery::TableFile;

/// What the app keeps on disk: the Benchmark table and the games behind each Trend.
#[derive(Debug, Clone, Copy)]
pub struct OnDisk<'a> {
    pub benchmark_table: &'a TableFile,
    pub history: &'a GameHistory,
}
