//! The part of the Benchmark table the Build reads, beside its `cells`.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use super::champion_key::champion_key;
use super::damage_profile::{DamageProfile, ProfileBucket};
use super::enemy_magic_share::enemy_magic_share;
use super::item_candidate::ItemCandidate;
use super::item_name::ItemName;
use super::threats::Threats;
use crate::game_inputs::Role;

const ICON_URL: &str = "https://ddragon.leagueoflegends.com/cdn";
// Item slots, the trinket aside.
const SLOTS: usize = 6;
// The share of magic damage from which a champion penetrates magic resist, not armor.
const MAGIC_DAMAGE_DEALER: f64 = 0.5;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ItemTable {
    damage_profile: Option<DamageProfile>,
    #[serde(default)]
    item_candidates: Vec<CandidatesEntry>,
    data_dragon_version: Option<String>,
    // By the id as a string: a flattened map cannot read its keys as numbers.
    #[serde(default)]
    item_names: HashMap<String, ItemName>,
    /// Data Dragon's tags by id, such as `Armor`; empty in a table built before them.
    #[serde(default)]
    item_tags: HashMap<String, Vec<String>>,
    /// Every boots id past the first tier; empty in a table built before they were listed.
    #[serde(default)]
    boots: HashSet<u32>,
    /// Every completed item id; empty in a table built before they were listed.
    #[serde(default)]
    completed: HashSet<u32>,
    threats: Option<Threats>,
    #[serde(default)]
    counters: Counters,
}

#[derive(Debug, Default, Deserialize)]
struct Counters {
    anti_heal: Vec<u32>,
    armor_penetration: Vec<u32>,
    magic_penetration: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct CandidatesEntry {
    champion: String,
    role: String,
    physical_heavy: Vec<CandidateEntry>,
    mixed: Vec<CandidateEntry>,
    magic_heavy: Vec<CandidateEntry>,
    usual_builds: Option<UsualBuilds>,
}

#[derive(Debug, Deserialize)]
struct UsualBuilds {
    physical_heavy: Vec<u32>,
    mixed: Vec<u32>,
    magic_heavy: Vec<u32>,
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
    /// The bucket of enemies, each a champion with the items it holds, `None` in a file
    /// with no Damage profile. A champion with no share counts as all physical, as the
    /// collector counts it.
    pub(crate) fn bucket_of<'a>(
        &self,
        enemies: impl Iterator<Item = (&'a str, Vec<u32>)>,
    ) -> Option<ProfileBucket> {
        let profile = self.damage_profile.as_ref()?;
        let shares = enemies
            .map(|(champion, items)| enemy_magic_share(self, profile.share_of(champion), &items));
        Some(profile.bucket_of(shares))
    }

    /// The boots and completed items of the list for `bucket`, largest share first. No
    /// list for the champion and role gives none: see ADR 0004.
    pub(crate) fn candidates(
        &self,
        champion: &str,
        role: Role,
        bucket: ProfileBucket,
    ) -> Vec<ItemCandidate> {
        let Some(entry) = self.entry(champion, role) else {
            return Vec::new();
        };
        let list = match bucket {
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

    /// The completed items games of the champion and role buy together against `bucket`,
    /// most bought first; empty in a table built before them.
    pub(crate) fn usual_build(&self, champion: &str, role: Role, bucket: ProfileBucket) -> &[u32] {
        let Some(builds) = self
            .entry(champion, role)
            .and_then(|entry| entry.usual_builds.as_ref())
        else {
            return &[];
        };
        match bucket {
            ProfileBucket::PhysicalHeavy => &builds.physical_heavy,
            ProfileBucket::Mixed => &builds.mixed,
            ProfileBucket::MagicHeavy => &builds.magic_heavy,
        }
    }

    /// For each Threat `enemies` are heavy in, the items that counter it: Grievous Wounds
    /// for healing, and penetration of the player's kind of damage for mitigation. A
    /// player with no magic share gets no penetration: a wrong kind is worse than none.
    pub(crate) fn counters_against(
        &self,
        enemies: &[&str],
        player_magic_share: Option<f64>,
    ) -> Vec<&[u32]> {
        let Some(threats) = &self.threats else {
            return Vec::new();
        };
        let counters = &self.counters;
        let mut wanted: Vec<&[u32]> = Vec::new();
        if threats.healing.heavy(enemies.iter().copied()) {
            wanted.push(&counters.anti_heal);
        }
        if let Some(share) = player_magic_share {
            if threats.mitigation.heavy(enemies.iter().copied()) {
                wanted.push(if share >= MAGIC_DAMAGE_DEALER {
                    &counters.magic_penetration
                } else {
                    &counters.armor_penetration
                });
            }
        }
        wanted
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

    /// Empty in a table built without tags.
    pub(crate) fn tags_of(&self, item: u32) -> &[String] {
        self.item_tags
            .get(&item.to_string())
            .map_or(&[], Vec::as_slice)
    }

    /// The item's icon on the Data Dragon CDN, at the version the file was built with.
    pub(crate) fn icon_url(&self, item: u32) -> Option<String> {
        let version = self.data_dragon_version.as_deref()?;
        Some(format!("{ICON_URL}/{version}/img/item/{item}.png"))
    }

    fn entry(&self, champion: &str, role: Role) -> Option<&CandidatesEntry> {
        let champion = champion_key(champion);
        self.item_candidates.iter().find(|entry| {
            champion_key(&entry.champion) == champion && Role::parse(&entry.role) == Some(role)
        })
    }
}
