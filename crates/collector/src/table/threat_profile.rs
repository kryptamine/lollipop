use std::collections::{BTreeMap, HashMap};

use super::average::quantile;
use super::benchmark_file::ThreatEntry;
use super::match_items::MatchItems;
use super::reworks::Reworks;
use super::threat::Threat;

// The top quarter of teams: a Counter item takes a Build slot, so it must be needed.
const HEAVY_SHARE: f64 = 0.75;

/// Per champion, a Threat per minute of game; a team's is the sum of its five. A team
/// at `heavy_from` or above is among the quarter of stored teams with the most.
pub(super) struct ThreatProfile {
    per_minute: BTreeMap<String, f64>,
    heavy_from: f64,
}

impl ThreatProfile {
    /// `None` with no archived match to measure.
    pub(super) fn of(matches: &[MatchItems], reworks: &Reworks, threat: Threat) -> Option<Self> {
        let mut sums: HashMap<&str, (u64, f64)> = HashMap::new();
        // A game with no length gives no amount a minute, so its teams count nowhere.
        let timed: Vec<&MatchItems> = matches.iter().filter(|game| game.minutes > 0.0).collect();
        for game in &timed {
            for participant in &game.participants {
                if reworks.drops(&participant.champion, game.patch) {
                    continue;
                }
                let sum = sums.entry(&participant.champion).or_default();
                sum.0 += threat.of(participant);
                sum.1 += game.minutes;
            }
        }
        let per_minute: BTreeMap<String, f64> = sums
            .into_iter()
            .map(|(champion, (amount, minutes))| {
                (champion.to_string(), tenth(float(amount) / minutes))
            })
            .collect();
        let mut teams: Vec<f64> = timed
            .iter()
            .flat_map(|game| {
                game.teams().map(|team| {
                    let members = game
                        .participants
                        .iter()
                        .filter(|member| member.team == team);
                    members
                        .filter_map(|member| per_minute.get(&member.champion))
                        .sum::<f64>()
                })
            })
            .collect();
        teams.sort_by(f64::total_cmp);
        let heavy_from = tenth(quantile(&teams, HEAVY_SHARE)?);
        Some(Self {
            per_minute,
            heavy_from,
        })
    }

    pub(super) fn entry(self) -> ThreatEntry {
        ThreatEntry {
            per_minute: self.per_minute,
            heavy_from: self.heavy_from,
        }
    }
}

// A tenth: the app sums five of these, and the bound is compared to that sum.
fn tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a Threat summed over a Patch window is far below 2^52"
)]
const fn float(amount: u64) -> f64 {
    amount as f64
}
