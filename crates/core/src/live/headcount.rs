use serde::Serialize;

use super::clock_text::countdown;
use super::payload::{AllGameData, Player};
use super::place::Place;
use super::spawn::Spawn;
use super::window_objective::WindowObjective;

/// The champions each side has alive, while one side has more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Headcount {
    pub allies: usize,
    pub enemies: usize,
    /// Until enough of the short side has respawned to even the sides, as `m:ss`, if no
    /// one else dies meanwhile.
    pub lead_ends_in: String,
    /// The soonest one up or spawning before the lead ends.
    pub objective: Option<WindowObjective>,
}

/// `None` while the sides are even, and in a game with one side empty.
pub(super) fn headcount_of(
    data: &AllGameData,
    player: &Player,
    spawns: &[Spawn],
) -> Option<Headcount> {
    let allies: Vec<&Player> = data.allies_of(player).collect();
    let enemies: Vec<&Player> = data.enemies_of(player).collect();
    if allies.is_empty() || enemies.is_empty() {
        return None;
    }
    let alive = |side: &[&Player]| side.iter().filter(|p| !p.is_dead).count();
    let (ours, theirs) = (alive(&allies), alive(&enemies));
    let short = if ours < theirs { &allies } else { &enemies };
    let lead = ours.abs_diff(theirs);
    if lead == 0 {
        return None;
    }
    let mut respawns: Vec<f64> = short
        .iter()
        .filter(|p| p.is_dead)
        .map(|p| p.respawn_timer)
        .collect();
    respawns.sort_by(f64::total_cmp);
    // A side of fewer than five can be short more champions than it has dead.
    let ends_in = respawns
        .get(lead - 1)
        .or_else(|| respawns.last())
        .copied()?;
    let now = data.game_time();
    let objective = spawns
        .iter()
        .filter(|spawn| spawn.at <= now + ends_in)
        .find_map(|spawn| {
            Some(WindowObjective {
                objective: contested(spawn.objective)?,
                spawns_in: (spawn.at > now).then(|| countdown(spawn.at - now)),
            })
        });
    Some(Headcount {
        allies: ours,
        enemies: theirs,
        lead_ends_in: countdown(ends_in),
        objective,
    })
}

/// Voidgrubs and Herald have no respawn the schedule follows, so whether they are up
/// is not known.
fn contested(objective: &str) -> Option<Place> {
    match objective {
        "Drake" => Some(Place::Drake),
        "Elder" => Some(Place::Elder),
        "Baron" => Some(Place::Baron),
        _ => None,
    }
}
