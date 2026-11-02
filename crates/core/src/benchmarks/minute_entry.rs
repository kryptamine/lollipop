use serde::Deserialize;

use super::benchmarks_at::BenchmarksAt;
use super::scores_at::ScoresAt;

const MINIMUM_N: u32 = 100;
pub(super) const SECONDS_PER_MINUTE: f64 = 60.0;

#[derive(Debug, Deserialize)]
pub(super) struct MinuteEntry {
    pub(super) minute: u32,
    cs: f64,
    total_gold: f64,
    level: f64,
    n: u32,
    #[serde(flatten)]
    scores: Option<ScoresEntry>,
}

#[derive(Debug, Deserialize)]
struct ScoresEntry {
    kills: f64,
    deaths: f64,
    assists: f64,
    scores_n: u32,
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
        let scores = self.enough_scores().zip(after.enough_scores());
        BenchmarksAt {
            cs: blend(self.cs, after.cs),
            gold: blend(self.total_gold, after.total_gold),
            level: blend(self.level, after.level),
            scores: scores.map(|(from, to)| ScoresAt {
                kills: blend(from.kills, to.kills),
                deaths: blend(from.deaths, to.deaths),
                assists: blend(from.assists, to.assists),
            }),
        }
    }

    // Scores come only from games kept since they were counted, so they thin out apart.
    fn enough_scores(&self) -> Option<&ScoresEntry> {
        self.scores.as_ref().filter(|s| s.scores_n >= MINIMUM_N)
    }
}
