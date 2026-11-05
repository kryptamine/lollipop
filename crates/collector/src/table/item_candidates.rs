use std::collections::BTreeMap;

use super::benchmark_file::{ItemCandidatesEntry, UsualBuildsEntry};
use super::candidate_list::candidate_list;
use super::damage_profile::DamageProfile;
use super::first_purchase::FirstPurchase;
use super::item_catalog::ItemCatalog;
use super::match_items::MatchItems;
use super::profile_bucket::ProfileBucket;
use super::reworks::Reworks;
use super::usual_build::usual_build;

// Riot leaves `teamPosition` empty on a game with no role, and no list has that.
const ROLES: [&str; 5] = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

type BucketGames<'a> = [Vec<&'a [FirstPurchase]>; 3];

/// One entry per champion and role over every tier. A bucket under `min_games` takes the
/// list pooled over buckets, and a pool under it gives no entry: see ADR 0004.
pub(super) fn item_candidates(
    matches: &[MatchItems],
    profile: &DamageProfile,
    reworks: &Reworks,
    catalog: &ItemCatalog,
    min_games: usize,
) -> Vec<ItemCandidatesEntry> {
    let mut groups: BTreeMap<(&str, &str), BucketGames<'_>> = BTreeMap::new();
    for game in matches {
        for player in &game.participants {
            if !ROLES.contains(&player.role.as_str()) || reworks.drops(&player.champion, game.patch)
            {
                continue;
            }
            let buckets = groups.entry((&player.champion, &player.role)).or_default();
            if let Some(games) = buckets.get_mut(profile.facing(game, player).index()) {
                games.push(&player.purchases);
            }
        }
    }
    groups
        .into_iter()
        .filter_map(|((champion, role), buckets)| {
            let pooled: Vec<&[FirstPurchase]> = buckets.iter().flatten().copied().collect();
            if pooled.len() < min_games {
                return None;
            }
            let games: [&[&[FirstPurchase]]; 3] = ProfileBucket::ALL.map(|bucket| {
                let own = buckets
                    .get(bucket.index())
                    .filter(|own| own.len() >= min_games);
                own.map_or(pooled.as_slice(), Vec::as_slice)
            });
            let [physical_games, mixed_games, magic_games] = games;
            let [physical_heavy, mixed, magic_heavy] =
                games.map(|games| candidate_list(games, catalog));
            let usual = |games, list: &[_]| usual_build(games, list, catalog, min_games);
            Some(ItemCandidatesEntry {
                champion: champion.to_string(),
                role: role.to_string(),
                usual_builds: UsualBuildsEntry {
                    physical_heavy: usual(physical_games, &physical_heavy),
                    mixed: usual(mixed_games, &mixed),
                    magic_heavy: usual(magic_games, &magic_heavy),
                },
                physical_heavy,
                mixed,
                magic_heavy,
            })
        })
        .collect()
}
