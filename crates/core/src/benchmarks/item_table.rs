//! The part of the Benchmark table item Advice reads, beside its `cells`.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::champion_key::champion_key;
use super::damage_profile::{DamageProfile, ProfileBucket};
use super::item_candidate::ItemCandidate;
use super::item_name::ItemName;
use crate::game_inputs::Role;

const ICON_URL: &str = "https://ddragon.leagueoflegends.com/cdn";

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
    /// The transforms that hold `item`.
    #[serde(default)]
    owned_as: Vec<u32>,
    #[serde(default)]
    builds_from: Vec<u32>,
}

impl ItemTable {
    /// The list for the bucket of the enemy team, less what `owned` holds, itself or
    /// transformed, and less every boots once it holds a pair. No list for the champion
    /// and role gives none: see ADR 0004.
    pub(crate) fn candidates<'a>(
        &self,
        champion: &str,
        role: Role,
        enemies: impl Iterator<Item = &'a str>,
        owned: &[u32],
    ) -> Vec<ItemCandidate<'_>> {
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
        let excluded = |candidate: &&CandidateEntry| {
            owned.contains(&candidate.item)
                || candidate.owned_as.iter().any(|item| owned.contains(item))
                || self.another_pair(candidate.item, owned)
        };
        list.iter()
            .filter(|candidate| !excluded(candidate))
            .map(|candidate| ItemCandidate {
                item: candidate.item,
                share: candidate.share,
                minute: candidate.minute,
                builds_from: &candidate.builds_from,
                name: self.name_of(candidate.item),
            })
            .collect()
    }

    /// Whether `item` is boots `owned` cannot take, since it already holds a pair.
    pub(crate) fn another_pair(&self, item: u32, owned: &[u32]) -> bool {
        self.boots.contains(&item)
            && !owned.contains(&item)
            && owned.iter().any(|own| self.boots.contains(own))
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
