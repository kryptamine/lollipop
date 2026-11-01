use serde::Deserialize;

use super::benchmarks_at::BenchmarksAt;

const MINIMUM_N: u32 = 100;
pub(super) const SECONDS_PER_MINUTE: f64 = 60.0;

#[derive(Debug, Deserialize)]
pub(super) struct MinuteEntry {
    pub(super) minute: u32,
    cs: f64,
    total_gold: f64,
    level: f64,
    n: u32,
}

impl MinuteEntry {
    pub(super) fn second(&self) -> f64 {
        f64::from(self.minute) * SECONDS_PER_MINUTE
    }

    pub(super) const fn with_enough_games(&self) -> Option<&Self> {
        if self.n >= MINIMUM_N {
            Some(self)
        } else {
            None
        }
    }

    pub(super) fn blended_with(&self, after: &Self, share: f64) -> BenchmarksAt {
        let blend = |from: f64, to: f64| (to - from).mul_add(share, from);
        BenchmarksAt {
            cs: blend(self.cs, after.cs),
            gold: blend(self.total_gold, after.total_gold),
            level: blend(self.level, after.level),
        }
    }
}
