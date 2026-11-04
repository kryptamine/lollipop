use super::event_name::EventName;
use super::payload::{AllGameData, Player};

// A champion kill is the only sign of a fight Live Client Data gives.
const FIGHT_LASTS: f64 = 20.0;

/// Whether the player, alive, killed or assisted in a champion kill within the last 20 s.
/// A dead player is out of the fight, and one respawned since is in base.
pub(super) fn fighting(data: &AllGameData, player: &Player) -> bool {
    fought_within(data, player, FIGHT_LASTS)
}

/// As `fighting`, over the last `seconds`.
pub(super) fn fought_within(data: &AllGameData, player: &Player, seconds: f64) -> bool {
    let now = data.game_time();
    !player.is_dead
        && data
            .events_named(EventName::ChampionKill)
            .filter(|kill| kill.time().is_some_and(|at| now - at < seconds))
            .any(|kill| {
                kill.killer_name
                    .iter()
                    .chain(&kill.assisters)
                    .any(|name| player.goes_by(name))
            })
}
