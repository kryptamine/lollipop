use super::scores_at::ScoresAt;

/// One Cell's Benchmarks at an exact moment of game time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchmarksAt {
    pub cs: f64,
    pub gold: f64,
    pub level: f64,
    /// `None` until enough games kept with scores reach this minute.
    pub scores: Option<ScoresAt>,
}
