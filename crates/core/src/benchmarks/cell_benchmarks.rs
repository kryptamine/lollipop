use serde::Deserialize;

use super::benchmarks_at::BenchmarksAt;

const MINIMUM_N: u32 = 100;
const SECONDS_PER_MINUTE: f64 = 60.0;

/// One Cell's minutes, in the order the build writes them: ascending.
#[derive(Debug)]
pub struct CellBenchmarks {
    minutes: Vec<MinuteEntry>,
}

#[derive(Debug, Deserialize)]
pub(super) struct MinuteEntry {
    minute: u32,
    cs: f64,
    total_gold: f64,
    level: f64,
    n: u32,
}

impl CellBenchmarks {
    pub(super) const fn new(minutes: Vec<MinuteEntry>) -> Self {
        Self { minutes }
    }

    /// Interpolates between the minutes around `game_time`. `None` when either is
    /// missing or thin: half of a blend would still rest on too few games.
    #[must_use]
    pub fn at(&self, game_time: f64) -> Option<BenchmarksAt> {
        let next = self.minutes.partition_point(|m| m.second() <= game_time);
        let before = self
            .minutes
            .get(next.checked_sub(1)?)?
            .with_enough_games()?;
        if game_time <= before.second() {
            return Some(before.blended_with(before, 0.0));
        }
        let after = self.minutes.get(next)?.with_enough_games()?;
        if after.minute.checked_sub(before.minute)? != 1 {
            return None;
        }
        let share = (game_time - before.second()) / SECONDS_PER_MINUTE;
        Some(before.blended_with(after, share))
    }
}

impl MinuteEntry {
    fn second(&self) -> f64 {
        f64::from(self.minute) * SECONDS_PER_MINUTE
    }

    const fn with_enough_games(&self) -> Option<&Self> {
        if self.n >= MINIMUM_N {
            Some(self)
        } else {
            None
        }
    }

    fn blended_with(&self, after: &Self, share: f64) -> BenchmarksAt {
        let blend = |from: f64, to: f64| (to - from).mul_add(share, from);
        BenchmarksAt {
            cs: blend(self.cs, after.cs),
            gold: blend(self.total_gold, after.total_gold),
            level: blend(self.level, after.level),
        }
    }
}
