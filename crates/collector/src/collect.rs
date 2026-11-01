use std::convert::Infallible;

use crate::kept_match::KeptMatch;
use crate::match_payload::{MatchPayload, RANKED_SOLO_QUEUE};
use crate::riot_api::RiotApi;
use crate::seed_walk::SeedWalk;
use crate::store::{PendingMatch, Store};
use crate::timeline_payload::TimelinePayload;
use crate::{Clock, CollectError, Config};

const MATCHES_PER_SEED: usize = 3;
const IDS_PER_SEED: u32 = 20;
// About the Patch window: 35 days spent 17% of fetches on patches already out of it.
const LOOKBACK_MS: u64 = 28 * 24 * 60 * 60 * 1000;

/// Runs until a failure a restart has to deal with, and resumes from `config.database`.
pub async fn collect(config: &Config, clock: &impl Clock) -> CollectError {
    match run(config, clock).await {
        Ok(never) => match never {},
        Err(error) => error,
    }
}

async fn run(config: &Config, clock: &impl Clock) -> Result<Infallible, CollectError> {
    let mut store = Store::open(&config.database)?;
    let api = RiotApi::new(config, clock)?;
    let mut walk = SeedWalk::new(config.tier);
    loop {
        process_pending(&api, &mut store).await?;
        let (tier, seeds) = walk.next_seeds(&api, &store).await?;
        for puuid in seeds {
            let ids = match_ids(&api, &store, &puuid).await?;
            store.enqueue(tier.label(), pick_unvisited(&store, ids)?)?;
            process_pending(&api, &mut store).await?;
        }
        // A dropped match this old cannot come back, as ids reach only as far as the lookback.
        store.forget_dropped_before(api.now_ms().saturating_sub(LOOKBACK_MS))?;
    }
}

async fn match_ids(
    api: &RiotApi<'_, impl Clock>,
    store: &Store,
    puuid: &str,
) -> Result<Vec<String>, CollectError> {
    let start_time_s = api.now_ms().saturating_sub(LOOKBACK_MS) / 1000;
    let path = format!(
        "/lol/match/v5/matches/by-puuid/{puuid}/ids\
         ?queue={RANKED_SOLO_QUEUE}&type=ranked&startTime={start_time_s}&count={IDS_PER_SEED}"
    );
    let ids = api.regional(store, "match-v5:ids", &path).await?;
    Ok(ids.unwrap_or_default())
}

fn pick_unvisited(store: &Store, ids: Vec<String>) -> Result<Vec<String>, CollectError> {
    let mut unvisited = Vec::new();
    for id in ids {
        if !store.is_visited(&id)? {
            unvisited.push(id);
        }
    }
    fastrand::shuffle(&mut unvisited);
    unvisited.truncate(MATCHES_PER_SEED);
    Ok(unvisited)
}

async fn process_pending(
    api: &RiotApi<'_, impl Clock>,
    store: &mut Store,
) -> Result<(), CollectError> {
    while let Some(pending) = store.next_pending()? {
        match fetch_kept(api, store, &pending).await {
            Ok(Some(kept)) => store.keep(&kept)?,
            Ok(None) => store.drop_match(&pending.match_id, api.now_ms())?,
            Err(error) if error.only_costs_the_match() => {
                store.drop_match(&pending.match_id, api.now_ms())?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

async fn fetch_kept(
    api: &RiotApi<'_, impl Clock>,
    store: &Store,
    pending: &PendingMatch,
) -> Result<Option<KeptMatch>, CollectError> {
    let id = &pending.match_id;
    let game_path = format!("/lol/match/v5/matches/{id}");
    let game: Option<MatchPayload> = api.regional(store, "match-v5:match", &game_path).await?;
    let Some(game) = game else {
        return Ok(None);
    };
    let Some(patch) = game.patch_if_kept(store.patch_window_floor()?) else {
        return Ok(None);
    };
    let timeline_path = format!("/lol/match/v5/matches/{id}/timeline");
    let timeline: Option<TimelinePayload> = api
        .regional(store, "match-v5:timeline", &timeline_path)
        .await?;
    Ok(timeline.map(|timeline| KeptMatch::new(pending, patch, game, &timeline)))
}
