use std::collections::HashMap;

use crate::league_entries::{ApexLeague, LeagueEntry};
use crate::riot_api::RiotApi;
use crate::store::Store;
use crate::{Clock, CollectError, SeedTier};

const SEEDS_PER_TURN: usize = 10;
const DIVISIONS: [&str; 4] = ["I", "II", "III", "IV"];
const APEX_LEAGUES: [(&str, &str); 3] = [
    ("league-v4:challengerleagues", "challengerleagues"),
    ("league-v4:grandmasterleagues", "grandmasterleagues"),
    ("league-v4:masterleagues", "masterleagues"),
];
// Above any division's page count. An empty page lowers it for the rest of the run.
const FIRST_PAGE_CAP: u32 = 1024;

/// Takes the tiers in turn, resuming the turn after the last one a previous run began.
pub(crate) struct SeedWalk {
    owner_tier: SeedTier,
    page_caps: HashMap<(SeedTier, &'static str), u32>,
}

impl SeedWalk {
    pub(crate) fn new(owner_tier: SeedTier) -> Self {
        Self {
            owner_tier,
            page_caps: HashMap::new(),
        }
    }

    /// A sample of one tier's seed players, with their PUUIDs.
    pub(crate) async fn next_seeds(
        &mut self,
        api: &RiotApi<'_, impl Clock>,
        store: &Store,
    ) -> Result<(SeedTier, Vec<String>), CollectError> {
        let tier = self.next_tier(store)?;
        store.set_last_tier(tier.label())?;
        let entries = if tier == SeedTier::MasterPlus {
            apex_entries(api, store).await?
        } else {
            self.random_page(api, store, tier).await?
        };
        let mut seeds: Vec<String> = entries.into_iter().filter_map(|e| e.puuid).collect();
        fastrand::shuffle(&mut seeds);
        seeds.truncate(SEEDS_PER_TURN);
        Ok((tier, seeds))
    }

    fn next_tier(&self, store: &Store) -> Result<SeedTier, CollectError> {
        let tiers = self.owner_tier.with_neighbours();
        let last = store.last_tier()?;
        let after_last = tiers
            .iter()
            .position(|tier| Some(tier.label()) == last.as_deref())
            .map_or(0, |position| position + 1);
        let next = tiers.iter().cycle().nth(after_last);
        Ok(next.copied().unwrap_or(self.owner_tier))
    }

    // Pages are sorted by LP, so a random page avoids sampling only a division's top.
    async fn random_page(
        &mut self,
        api: &RiotApi<'_, impl Clock>,
        store: &Store,
        tier: SeedTier,
    ) -> Result<Vec<LeagueEntry>, CollectError> {
        let division = fastrand::choice(DIVISIONS).unwrap_or("I");
        let cap = self
            .page_caps
            .entry((tier, division))
            .or_insert(FIRST_PAGE_CAP);
        let page = fastrand::u32(1..=*cap);
        let path = format!(
            "/lol/league/v4/entries/RANKED_SOLO_5x5/{}/{division}?page={page}",
            tier.label()
        );
        let entries: Vec<LeagueEntry> = api
            .platform(store, "league-v4:entries", &path)
            .await?
            .unwrap_or_default();
        if entries.is_empty() {
            *cap = (page - 1).max(1);
        }
        Ok(entries)
    }
}

async fn apex_entries(
    api: &RiotApi<'_, impl Clock>,
    store: &Store,
) -> Result<Vec<LeagueEntry>, CollectError> {
    let mut pool = Vec::new();
    for (call, league) in APEX_LEAGUES {
        let path = format!("/lol/league/v4/{league}/by-queue/RANKED_SOLO_5x5");
        let body: Option<ApexLeague> = api.platform(store, call, &path).await?;
        pool.extend(body.map_or_else(Vec::new, |league| league.entries));
    }
    Ok(pool)
}
