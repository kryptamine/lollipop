//! Reads the Benchmark table file that `lollipop-collector build-table` writes.

mod benchmark_table;
mod benchmarks_at;
mod cell_benchmarks;
mod champion_key;
mod damage_profile;
mod enemy_magic_share;
mod item_candidate;
mod item_name;
mod item_table;
mod minute_entry;
mod scores_at;
mod threats;

pub use benchmark_table::BenchmarkTable;
pub(crate) use benchmark_table::SCHEMA_VERSION;
pub use benchmarks_at::BenchmarksAt;
pub use cell_benchmarks::CellBenchmarks;
pub(crate) use damage_profile::ProfileBucket;
pub(crate) use item_candidate::ItemCandidate;
pub(crate) use item_table::ItemTable;
pub use scores_at::ScoresAt;
