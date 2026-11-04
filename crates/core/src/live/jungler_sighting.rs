use serde::Serialize;

use super::clock_text::{ago, countdown};
use super::enemy_watch::EnemyWatch;
use super::event_name::EventName;
use super::payload::{AllGameData, Event, Player};
use super::place::Place;
use super::whereabouts::Whereabouts;

// Where a jungler was this long ago says little about where they are now.
const KEPT_FOR: f64 = 90.0;

/// The enemy jungler's last known whereabouts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JunglerSighting {
    pub champion: String,
    pub whereabouts: Whereabouts,
}

/// `None` with no enemy in the jungle, before they are first seen and once the sighting
/// is old. Live Client Data has no positions: a kill, an objective or a trip to base
/// they took part in is the only sign.
pub(super) fn sighting_of(
    data: &AllGameData,
    player: &Player,
    watch: &EnemyWatch,
) -> Option<JunglerSighting> {
    let jungler = data
        .enemies_of(player)
        .find(|enemy| enemy.position.as_deref() == Some("JUNGLE"))?;
    let champion = jungler.champion_name.clone()?;
    if jungler.is_dead {
        let back_in = countdown(jungler.respawn_timer);
        return Some(JunglerSighting {
            champion,
            whereabouts: Whereabouts::Dead { back_in },
        });
    }
    let (at, place) = last_seen(data, jungler, watch)?;
    let passed = data.game_time() - at;
    (passed <= KEPT_FOR).then(|| JunglerSighting {
        champion,
        whereabouts: Whereabouts::Seen {
            place,
            ago: ago(passed),
        },
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
        EventName::HeraldKill => Place::Herald,
        EventName::BaronKill => Place::Baron,
        _ => return None,
    };
    Some((event.time()?, place))
}
