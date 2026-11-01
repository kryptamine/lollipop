use std::collections::BTreeMap;

use lollipop_core::game_inputs::Patch;

use super::aligned_minute::align_to_minutes;
use super::average::{mean, median};
use super::benchmark_file::{CellEntry, MinuteEntry};
use super::cell_drift::CellDrift;
use super::drift_comparison::DriftComparison;
use super::minute_samples::MinuteSamples;
use super::patch_medians::PatchMedians;
use crate::minute_values::MinuteValues;
use crate::seed_tier::SeedTier;

// The minute a thin Cell's source is chosen at, and the fewest games a source may have then.
const SOURCE_MINUTE: usize = 20;
const MINIMUM_N: usize = 100;

/// The samples of one champion and role, split by tier and patch.
pub(super) struct CellSamples {
    champion: String,
    role: String,
    by_tier_and_patch: BTreeMap<(SeedTier, Patch), Vec<MinuteSamples>>,
}

impl CellSamples {
    pub(super) fn new(champion: &str, role: &str) -> Self {
        Self {
            champion: champion.to_string(),
            role: role.to_string(),
            by_tier_and_patch: BTreeMap::new(),
        }
    }

    pub(super) fn holds(&self, champion: &str, role: &str) -> bool {
        self.champion == champion && self.role == role
    }

    pub(super) fn add(&mut self, tier: SeedTier, patch: Patch, frames: &[MinuteValues]) {
        let minutes = self.by_tier_and_patch.entry((tier, patch)).or_default();
        for aligned in align_to_minutes(frames) {
            if minutes.len() <= aligned.minute {
                minutes.resize_with(aligned.minute + 1, MinuteSamples::default);
            }
            if let Some(samples) = minutes.get_mut(aligned.minute) {
                samples.add(&aligned);
            }
        }
    }

    /// A Cell's own games if minute 20 has enough, else with its adjacent tiers, else no
    /// entry. Every minute then reads that one source, so a Benchmark never jumps.
    pub(super) fn cell_entries(&self) -> Vec<CellEntry> {
        SeedTier::ALL
            .into_iter()
            .filter_map(|tier| {
                let own = [tier];
                let source = [own.as_slice(), tier.with_neighbours()]
                    .into_iter()
                    .find(|tiers| self.n_at_source_minute(tiers) >= MINIMUM_N)?;
                Some(CellEntry {
                    champion: self.champion.clone(),
                    role: self.role.clone(),
                    tier: tier.label(),
                    minutes: self.minute_entries(source),
                })
            })
            .collect()
    }

    /// Compares a Cell's own games only: a tier mix that differs by patch would move a
    /// pooled median with no change to the champion.
    pub(super) fn drifts(&self, comparison: DriftComparison) -> Vec<CellDrift> {
        SeedTier::ALL
            .into_iter()
            .filter_map(|tier| {
                Some(CellDrift {
                    champion: self.champion.clone(),
                    role: self.role.clone(),
                    tier: tier.label(),
                    older: self.medians_on(tier, comparison.older)?,
                    newer: self.medians_on(tier, comparison.newer)?,
                })
            })
            .filter(CellDrift::is_sharp)
            .collect()
    }

    fn medians_on(&self, tier: SeedTier, patch: Patch) -> Option<PatchMedians> {
        let samples = self
            .by_tier_and_patch
            .get(&(tier, patch))?
            .get(SOURCE_MINUTE)?;
        if samples.n() < MINIMUM_N {
            return None;
        }
        Some(PatchMedians {
            cs: median(&samples.cs)?,
            total_gold: median(&samples.total_gold)?,
        })
    }

    fn n_at_source_minute(&self, tiers: &[SeedTier]) -> usize {
        self.of_tiers(tiers)
            .filter_map(|minutes| minutes.get(SOURCE_MINUTE))
            .map(MinuteSamples::n)
            .sum()
    }

    fn minute_entries(&self, tiers: &[SeedTier]) -> Vec<MinuteEntry> {
        let last = self.of_tiers(tiers).map(Vec::len).max().unwrap_or(0);
        (0..last)
            .filter_map(|minute| {
                let mut pooled = MinuteSamples::default();
                for samples in self.of_tiers(tiers).filter_map(|all| all.get(minute)) {
                    pooled.extend(samples);
                }
                minute_entry(minute, &pooled)
            })
            .collect()
    }

    fn of_tiers<'a>(
        &'a self,
        tiers: &'a [SeedTier],
    ) -> impl Iterator<Item = &'a Vec<MinuteSamples>> {
        self.by_tier_and_patch
            .iter()
            .filter(|((tier, _), _)| tiers.contains(tier))
            .map(|(_, minutes)| minutes)
    }
}

fn minute_entry(minute: usize, pooled: &MinuteSamples) -> Option<MinuteEntry> {
    Some(MinuteEntry {
        minute,
        cs: rounded(median(&pooled.cs)?),
        total_gold: rounded(median(&pooled.total_gold)?),
        level: rounded(mean(&pooled.level)),
        n: pooled.n(),
    })
}

// Two decimals is finer than any Neutral zone, and keeps noise digits out of the file.
fn rounded(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
