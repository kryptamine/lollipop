use super::moment::Moment;
use super::payload::{AllGameData, Event};
use super::team::Team;

// The laning phase ends here when no turret has fallen yet.
const LANING_ENDS: f64 = 840.0;
// A team's drakes from which the Dragon Soul is in play; an earlier drake rarely
// changes the plan.
const SOUL_POINT: usize = 3;

/// A moment that changes the game plan.
pub(super) struct TurningPoint {
    pub(super) game_time: f64,
    pub(super) moment: Moment,
    /// `None` when the event names no team.
    pub(super) taken_by: Option<Team>,
}

/// Watches one game's polls for Turning points. What happened before its first poll,
/// as when the app starts mid-game, fires nothing.
#[derive(Default)]
pub(super) struct TurningPointWatch {
    seen_events: Option<usize>,
    laning_over: bool,
}

impl TurningPointWatch {
    /// The latest Turning point since the last poll.
    pub(super) fn seen(&mut self, data: &AllGameData) -> Option<TurningPoint> {
        let events = data.events();
        // 14:00 fires nothing, but a first turret after it is a later turret.
        let laning_over = data.game_time() >= LANING_ENDS
            || events.iter().any(|event| event.name() == "TurretKilled");
        let Some(seen) = self.seen_events.replace(events.len()) else {
            self.laning_over = laning_over;
            return None;
        };
        let mut latest = None;
        for (index, event) in events.iter().enumerate().skip(seen) {
            latest = self.of_event(data, index, event).or(latest);
        }
        self.laning_over |= laning_over;
        latest
    }

    /// Later turrets, Void Grubs, kills and drakes short of the soul point are no
    /// Turning point.
    fn of_event(
        &mut self,
        data: &AllGameData,
        index: usize,
        event: &Event,
    ) -> Option<TurningPoint> {
        let taken_by = data.taker_team(event);
        let moment = match event.name() {
            "TurretKilled" if !self.laning_over => {
                self.laning_over = true;
                Moment::FirstTurret
            }
            "DragonKill"
                if event.is_elder()
                    || taken_by.is_some_and(|team| drakes_of(data, index, team) >= SOUL_POINT) =>
            {
                Moment::Dragon(event.dragon_type().map(str::to_string))
            }
            "HeraldKill" => Moment::Herald,
            "BaronKill" => Moment::Baron,
            "InhibKilled" => Moment::Inhibitor,
            "Ace" => Moment::Ace,
            _ => return None,
        };
        Some(TurningPoint {
            game_time: event.time().unwrap_or_else(|| data.game_time()),
            moment,
            taken_by,
        })
    }
}

/// The drakes `team` holds once the event at `index` is counted, the Elder aside.
fn drakes_of(data: &AllGameData, index: usize, team: Team) -> usize {
    data.events()
        .iter()
        .take(index + 1)
        .filter(|event| event.name() == "DragonKill" && !event.is_elder())
        .filter(|event| data.taker_team(event) == Some(team))
        .count()
}
