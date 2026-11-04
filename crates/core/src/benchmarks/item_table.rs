//! The part of the Benchmark table the Build reads, beside its `cells`.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::champion_key::champion_key;
use super::damage_profile::{DamageProfile, ProfileBucket};
use super::item_candidate::ItemCandidate;
use super::item_name::ItemName;
use crate::game_inputs::Role;

const ICON_URL: &str = "https://ddragon.leagueoflegends.com/cdn";
// Item slots, the trinket aside.
const SLOTS: usize = 6;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ItemTable {
    damage_profile: Option<DamageProfile>,
    #[serde(default)]
    item_candidates: Vec<CandidatesEntry>,
    data_dragon_version: Option<String>,
    // By the id as a string: a flattened map cannot read its keys as numbers.
    #[serde(default)]
    item_names: HashMap<String, ItemName>,
    /// Every boots id past the first tier; empty in a table built before they were listed.
    #[serde(default)]
    boots: HashSet<u32>,
    /// Every completed item id; empty in a table built before they were listed.
    #[serde(default)]
    completed: HashSet<u32>,
}

#[derive(Debug, Deserialize)]
struct CandidatesEntry {
    champion: String,
    role: String,
    physical_heavy: Vec<CandidateEntry>,
    mixed: Vec<CandidateEntry>,
    magic_heavy: Vec<CandidateEntry>,
}

#[derive(Debug, Deserialize)]
struct CandidateEntry {
    item: u32,
    share: f64,
    minute: f64,
    #[serde(default)]
    owned_as: Vec<u32>,
    #[serde(default)]
    builds_from: Vec<u32>,
}

impl ItemTable {
    /// The boots and completed items of the list for the enemy team's bucket, largest
    /// share first. No list for the champion and role gives none: see ADR 0004.
    pub(crate) fn candidates<'a>(
        &self,
        champion: &str,
        role: Role,
        enemies: impl Iterator<Item = &'a str>,
    ) -> Vec<ItemCandidate> {
        let Some(profile) = &self.damage_profile else {
            return Vec::new();
        };
        let champion = champion_key(champion);
        let Some(entry) = self.item_candidates.iter().find(|entry| {
            champion_key(&entry.champion) == champion && Role::parse(&entry.role) == Some(role)
        }) else {
            return Vec::new();
        };
        let list = match profile.bucket_of(enemies) {
            ProfileBucket::PhysicalHeavy => &entry.physical_heavy,
            ProfileBucket::Mixed => &entry.mixed,
            ProfileBucket::MagicHeavy => &entry.magic_heavy,
        };
        let mut candidates: Vec<ItemCandidate> = list
            .iter()
            .filter(|candidate| {
                self.is_boots(candidate.item) || self.completed.contains(&candidate.item)
            })
            .map(|candidate| ItemCandidate {
                item: candidate.item,
                boots: self.is_boots(candidate.item),
                share: candidate.share,
                minute: candidate.minute,
                owned_as: candidate.owned_as.clone(),
                builds_from: candidate.builds_from.clone(),
            })
            .collect();
        candidates.sort_by(|a, b| b.share.total_cmp(&a.share));
        candidates
    }

    /// The inventory slots `owned` leaves without a completed item or boots.
    pub(crate) fn free_slots(&self, owned: &[u32]) -> usize {
        SLOTS.saturating_sub(self.finished(owned).len())
    }

    /// The completed items and boots `owned` holds.
    pub(crate) fn finished(&self, owned: &[u32]) -> Vec<u32> {
        owned
            .iter()
            .copied()
            .filter(|item| self.completed.contains(item) || self.is_boots(*item))
            .collect()
    }

    pub(crate) fn is_boots(&self, item: u32) -> bool {
        self.boots.contains(&item)
    }

    /// The share of the champion's damage to champions that is magic, `None` when the
    /// file has none for it.
    pub(crate) fn magic_share(&self, champion: &str) -> Option<f64> {
        self.damage_profile.as_ref()?.share_of(champion)
    }

    pub(crate) fn name_of(&self, item: u32) -> Option<&ItemName> {
        self.item_names.get(&item.to_string())
    }

    /// The item's icon on the Data Dragon CDN, at the version the file was built with.
    pub(crate) fn icon_url(&self, item: u32) -> Option<String> {
        let version = self.data_dragon_version.as_deref()?;
        Some(format!("{ICON_URL}/{version}/img/item/{item}.png"))
    }
}
