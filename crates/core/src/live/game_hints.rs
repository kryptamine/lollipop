use super::build_item::BuildItem;
use super::dragon_pit;
use super::objective_schedule::ObjectiveSchedule;
use super::payload::AllGameData;
use super::spawn::Spawn;
use super::timing_hint::TimingHint;
use crate::game_inputs::{ClientInputs, GameInputs};

// Draft, ranked solo, blind, ranked flex and Practice Tool. Swiftplay and anything
// unknown fall outside.
pub(super) const HINT_QUEUES: [u32; 5] = [400, 420, 430, 440, 3140];

const ELIXIR_OBJECTIVES: [&str; 2] = ["Baron", "Elder"];

/// One game's Timing hints. The schedule is picked once, as the game starts.
#[derive(Default)]
pub(super) struct GameHints {
    schedule: Option<&'static ObjectiveSchedule>,
}

impl GameHints {
    pub(super) fn start(inputs: &GameInputs) -> Self {
        Self {
            schedule: schedule_of(inputs),
        }
    }

    /// The soonest spawn first, `elixir` beside Baron and Elder.
    pub(super) fn at(&self, data: &AllGameData, elixir: Option<&BuildItem>) -> Vec<TimingHint> {
        let Some(schedule) = self.schedule else {
            return Vec::new();
        };
        let mut spawns: Vec<Spawn> = [
            dragon_pit::next_spawn(schedule, data),
            next_baron(schedule, data),
            Some(Spawn {
                objective: "Voidgrubs",
                at: schedule.voidgrubs_first,
            }),
            Some(Spawn {
                objective: "Herald",
                at: schedule.herald_first,
            }),
        ]
        .into_iter()
        .flatten()
        .collect();
        spawns.sort_by(|a, b| a.at.total_cmp(&b.at));
        spawns
            .iter()
            .filter_map(|spawn| spawn.hint_at(data.game_time()))
            .map(|mut hint| {
                if ELIXIR_OBJECTIVES.contains(&hint.objective) {
                    hint.elixir = elixir.cloned();
                }
                hint
            })
            .collect()
    }
}

fn schedule_of(inputs: &GameInputs) -> Option<&'static ObjectiveSchedule> {
    let ClientInputs::Read(client) = &inputs.client else {
        return None;
    };
    if !HINT_QUEUES.contains(&client.queue?) {
        return None;
    }
    ObjectiveSchedule::for_patch(client.patch?)
}

fn next_baron(schedule: &ObjectiveSchedule, data: &AllGameData) -> Option<Spawn> {
    let at = match data.events_named("BaronKill").last() {
        Some(kill) => kill.time()? + schedule.baron_respawn,
        None => schedule.baron_first,
    };
    Some(Spawn {
        objective: "Baron",
        at,
    })
}
