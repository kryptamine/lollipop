use std::collections::{BTreeMap, HashMap};

use super::benchmark_file::DamageProfileEntry;
use super::match_items::MatchItems;
use super::participant_items::ParticipantItems;
use super::profile_bucket::ProfileBucket;
use super::reworks::Reworks;

/// Per champion, magic over total damage dealt to champions; a team's profile is the sum
/// of its five shares. The bounds are terciles of that sum over every stored team.
pub(super) struct DamageProfile {
    magic_share: BTreeMap<String, f64>,
    mixed_from: f64,
    magic_heavy_from: f64,
}

impl DamageProfile {
    /// `None` with no archived match to measure.
    pub(super) fn of(matches: &[MatchItems], reworks: &Reworks) -> Option<Self> {
        let mut damage: HashMap<&str, (u64, u64)> = HashMap::new();
        for game in matches {
            for participant in &game.participants {
                if reworks.drops(&participant.champion, game.patch) {
                    continue;
                }
                let sums = damage.entry(&participant.champion).or_default();
                sums.0 += participant.magic_damage;
                sums.1 += participant.total_damage;
            }
        }
        let magic_share: BTreeMap<String, f64> = damage
            .into_iter()
            .filter(|(_, (_, total))| *total > 0)
            .map(|(champion, (magic, total))| (champion.to_string(), rounded(ratio(magic, total))))
            .collect();
        let mut profile = Self {
            magic_share,
            mixed_from: 0.0,
            magic_heavy_from: 0.0,
        };
        let mut sums: Vec<f64> = matches
            .iter()
            .flat_map(|game| teams(game).map(|team| profile.team_sum(game, team)))
            .collect();
        sums.sort_by(f64::total_cmp);
        profile.mixed_from = rounded(quantile(&sums, 1.0 / 3.0)?);
        profile.magic_heavy_from = rounded(quantile(&sums, 2.0 / 3.0)?);
        Some(profile)
    }

    /// The bucket of the team facing `player`. Each bound is the lowest sum of its bucket.
    pub(super) fn facing(&self, game: &MatchItems, player: &ParticipantItems) -> ProfileBucket {
        let enemies = teams(game).find(|team| *team != player.team);
        let sum = enemies.map_or(0.0, |team| self.team_sum(game, team));
        if sum >= self.magic_heavy_from {
            ProfileBucket::MagicHeavy
        } else if sum >= self.mixed_from {
            ProfileBucket::Mixed
        } else {
            ProfileBucket::PhysicalHeavy
        }
    }

    pub(super) fn entry(self) -> DamageProfileEntry {
        DamageProfileEntry {
            magic_share: self.magic_share,
            mixed_from: self.mixed_from,
            magic_heavy_from: self.magic_heavy_from,
        }
    }

    // A champion with no measured damage counts as all physical, as the app must too.
    fn team_sum(&self, game: &MatchItems, team: u32) -> f64 {
        let members = game
            .participants
            .iter()
            .filter(|member| member.team == team);
        let shares = members.filter_map(|member| self.magic_share.get(&member.champion));
        shares.sum()
    }
}

fn teams(game: &MatchItems) -> impl Iterator<Item = u32> + '_ {
    let mut seen: Vec<u32> = Vec::new();
    game.participants.iter().filter_map(move |participant| {
        let new = !seen.contains(&participant.team);
        seen.push(participant.team);
        new.then_some(participant.team)
    })
}

/// Linear between the two nearest of sorted `values`.
fn quantile(values: &[f64], share: f64) -> Option<f64> {
    let last = values.len().checked_sub(1)?;
    let position = share * count(last);
    let below = position.floor();
    let low = *values.get(index(below))?;
    let high = values.get(index(below) + 1).copied().unwrap_or(low);
    Some((high - low).mul_add(position - below, low))
}

// Three decimals: the app sums five of these, and the bounds are compared to that sum.
fn rounded(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "damage totals over a Patch window are far below 2^52"
)]
fn ratio(part: u64, whole: u64) -> f64 {
    part as f64 / whole as f64
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a count of teams is far below 2^52"
)]
const fn count(value: usize) -> f64 {
    value as f64
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a whole, non-negative position within the slice"
)]
const fn index(position: f64) -> usize {
    position as usize
}
