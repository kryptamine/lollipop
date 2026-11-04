use serde::Serialize;

use super::clock_text::{ago, countdown};
use super::enemy_watch::EnemyWatch;
use super::event_name::EventName;
use super::payload::{AllGameData, Event, Player};
use super::place::Place;
use super::safe_lane::SafeLane;
use super::whereabouts::Whereabouts;

// Where a jungler was this long ago says little about where they are now.
const KEPT_FOR: f64 = 90.0;
// CS moves in steps of 10, so a jungler farming can go a minute without a step.
const NO_FARM_FROM: f64 = 90.0;

/// What is known of the enemy jungler. Live Client Data has no positions: a kill, an
/// objective, a Recall, a respawn or their farm is the only sign.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JunglerSighting {
    pub champion: String,
    /// `None` before they are first seen, and once the sighting is old.
    pub whereabouts: Option<Whereabouts>,
    pub safe_lane: Option<SafeLane>,
    /// The time since their CS or level last rose, as `m:ss`: ganking, or in base.
    pub no_farm_for: Option<String>,
}

/// `None` with no enemy jungler, and while nothing is known of them.
pub(super) fn sighting_of(
    data: &AllGameData,
    player: &Player,
    watch: &EnemyWatch,
) -> Option<JunglerSighting> {
    let jungler = jungler_of(data, player)?;
    let champion = jungler.champion_name.clone()?;
    if jungler.is_dead {
        let back_in = countdown(jungler.respawn_timer);
        return Some(JunglerSighting {
            champion,
            whereabouts: Some(Whereabouts::Dead { back_in }),
            safe_lane: None,
            no_farm_for: None,
        });
    }
    let now = data.game_time();
    let seen = last_seen(data, jungler, watch).filter(|(at, _)| now - at <= KEPT_FOR);
    let no_farm_for = watch
        .farmed_at(jungler)
        .map(|at| now - at)
        .filter(|passed| *passed >= NO_FARM_FROM)
        .map(ago);
    if seen.is_none() && no_farm_for.is_none() {
        return None;
    }
    Some(JunglerSighting {
        champion,
        whereabouts: seen.map(|(at, place)| Whereabouts::Seen {
            place,
            ago: ago(now - at),
        }),
        safe_lane: seen.and_then(|(at, place)| SafeLane::after(place, now - at)),
        no_farm_for,
    })
}

/// By position, or by Smite where the payload gives no position.
fn jungler_of<'a>(data: &'a AllGameData, player: &'a Player) -> Option<&'a Player> {
    let by_position = data
        .enemies_of(player)
        .find(|enemy| enemy.position.as_deref() == Some("JUNGLE"));
    by_position.or_else(|| {
        let mut smiting = data
            .enemies_of(player)
            .filter(|enemy| enemy.has_summoner("SummonerSmite"));
        let jungler = smiting.next()?;
        smiting.next().is_none().then_some(jungler)
    })
}

fn last_seen(data: &AllGameData, jungler: &Player, watch: &EnemyWatch) -> Option<(f64, Place)> {
    let in_event = data
        .events()
        .iter()
        .rev()
        .find_map(|event| place_of(data, jungler, event));
    let in_base = watch.in_base_at(jungler).map(|at| (at, Place::Base));
    in_event
        .into_iter()
        .chain(in_base)
        .max_by(|a, b| a.0.total_cmp(&b.0))
}

/// Where `event` puts the jungler, when they killed or assisted in it.
fn place_of(data: &AllGameData, jungler: &Player, event: &Event) -> Option<(f64, Place)> {
    let took_part = event
        .killer_name
        .iter()
        .chain(&event.assisters)
        .any(|name| jungler.goes_by(name));
    if !took_part {
        return None;
    }
    let place = match event.name() {
        EventName::ChampionKill => {
            let victim = data.player_named(event.victim_name.as_deref()?)?;
            Place::of_position(victim.position.as_deref()?)?
        }
        EventName::DragonKill if event.is_elder() => Place::Elder,
        EventName::DragonKill => Place::Drake,
        EventName::HordeKill => Place::Voidgrubs,
        EventName::HeraldKill => Place::Herald,
        EventName::BaronKill => Place::Baron,
        _ => return None,
    };
    Some((event.time()?, place))
}
