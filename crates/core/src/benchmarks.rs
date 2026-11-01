//! Reads the Benchmark table file that `lollipop-collector build-table` writes.

mod benchmark_table;
mod benchmarks_at;
mod cell_benchmarks;
mod champion_key;
mod minute_entry;

pub use benchmark_table::BenchmarkTable;
pub(crate) use benchmark_table::SCHEMA_VERSION;
pub use benchmarks_at::BenchmarksAt;
pub use cell_benchmarks::CellBenchmarks;
