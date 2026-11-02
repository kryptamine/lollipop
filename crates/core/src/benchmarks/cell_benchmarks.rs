use super::benchmarks_at::BenchmarksAt;
use super::minute_entry::{MinuteEntry, SECONDS_PER_MINUTE};

/// One Cell's minutes, in the order the build writes them: ascending.
#[derive(Debug)]
pub struct CellBenchmarks {
    minutes: Vec<MinuteEntry>,
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

    /// The Benchmarks at the next whole minute, which change once a minute rather than
    /// on every poll. A whole minute reads as itself.
    #[must_use]
    pub fn at_minute_ahead(&self, game_time: f64) -> Option<BenchmarksAt> {
        self.at((game_time / SECONDS_PER_MINUTE).ceil() * SECONDS_PER_MINUTE)
    }
}
