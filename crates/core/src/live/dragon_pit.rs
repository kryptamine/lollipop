use super::objective_schedule::ObjectiveSchedule;
use super::payload::AllGameData;
use super::spawn::Spawn;
use super::team::Team;

const DRAKES_FOR_ELDER: u32 = 4;

#[derive(Default)]
struct DrakeCounts {
    order: u32,
    chaos: u32,
}

impl DrakeCounts {
    /// How many drakes `team` has, this one included.
    const fn add(&mut self, team: Team) -> u32 {
        let count = match team {
            Team::Order => &mut self.order,
            Team::Chaos => &mut self.chaos,
        };
        *count += 1;
        *count
    }
}

/// What the pit spawns next, rebuilt from the whole event list so a start mid-game needs
/// no history. `None` once a kill has no team: the Elder rule can no longer be applied.
pub(super) fn next_spawn(schedule: &ObjectiveSchedule, data: &AllGameData) -> Option<Spawn> {
    let mut counts = DrakeCounts::default();
    let mut next = drake(schedule.drake_first);
    for kill in data.events_named("DragonKill") {
        let team = data.killer_team(kill)?;
        let killed_at = kill.time()?;
        next = if kill.is_elder() {
            elder(killed_at + schedule.elder_respawn)
        } else if counts.add(team) >= DRAKES_FOR_ELDER {
            elder(killed_at + schedule.elder_after_fourth_drake)
        } else {
            drake(killed_at + schedule.drake_respawn)
        };
    }
    Some(next)
}

const fn drake(at: f64) -> Spawn {
    Spawn {
        objective: "Drake",
        at,
    }
}

const fn elder(at: f64) -> Spawn {
    Spawn {
        objective: "Elder",
        at,
    }
}
