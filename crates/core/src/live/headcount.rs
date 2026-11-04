use std::cmp::Ordering;

use serde::Serialize;

use super::clock_text::countdown;
use super::payload::{AllGameData, Player};
use super::spawn::Spawn;
use super::window_objective::WindowObjective;

// Voidgrubs and Herald have no respawn the schedule follows, so whether they are up
// is not known.
const CONTESTED: [&str; 3] = ["Drake", "Elder", "Baron"];

/// The champions each side has alive, while one side has more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Headcount {
    pub allies: usize,
    pub enemies: usize,
    /// Until the short side's next respawn, as `m:ss`. The other side can only gain
    /// meanwhile, so the lead lasts at least this long.
    pub lasts: String,
    /// The soonest one up or spawning before the lead may end.
    pub objective: Option<WindowObjective>,
}

/// `None` while the sides are even, and in a game with one side empty.
pub(super) fn headcount_of(
    data: &AllGameData,
    player: &Player,
    spawns: &[Spawn],
) -> Option<Headcount> {
    let team = player.team()?;
    let (allies, enemies): (Vec<&Player>, Vec<&Player>) = data
        .players()
        .iter()
        .filter(|p| p.team().is_some())
        .partition(|p| p.team() == Some(team));
    if allies.is_empty() || enemies.is_empty() {
        return None;
    }
    let alive = |side: &[&Player]| side.iter().filter(|p| !p.is_dead).count();
    let (ours, theirs) = (alive(&allies), alive(&enemies));
    let short = match ours.cmp(&theirs) {
        Ordering::Less => &allies,
        Ordering::Greater => &enemies,
        Ordering::Equal => return None,
    };
    let lasts = short
        .iter()
        .filter(|p| p.is_dead)
        .map(|p| p.respawn_timer)
        .min_by(f64::total_cmp)?;
    let now = data.game_time();
    let objective = spawns
        .iter()
        .filter(|spawn| CONTESTED.contains(&spawn.objective))
        .find(|spawn| spawn.at <= now + lasts)
        .map(|spawn| WindowObjective {
            objective: spawn.objective,
            spawns_in: (spawn.at > now).then(|| countdown(spawn.at - now)),
        });
    Some(Headcount {
        allies: ours,
        enemies: theirs,
        lasts: countdown(lasts),
        objective,
    })
}
