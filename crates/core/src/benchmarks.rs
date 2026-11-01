//! Reads the Benchmark table file that `lollipop-collector build-table` writes.

mod benchmark_table;
mod benchmarks_at;
mod cell_benchmarks;
mod champion_key;

pub use benchmark_table::BenchmarkTable;
pub use benchmarks_at::BenchmarksAt;
pub use cell_benchmarks::CellBenchmarks;
