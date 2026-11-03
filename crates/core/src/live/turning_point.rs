use super::moment::Moment;
use super::payload::{AllGameData, Event, Player};
use super::team::Team;

// The laning phase ends here when no turret has fallen yet.
const LANING_ENDS: f64 = 840.0;
const LEVELS: [u32; 3] = [6, 11, 16];

/// A moment that changes the game plan.
pub(super) struct TurningPoint {
    pub(super) game_time: f64,
    pub(super) moment: Moment,
    /// `None` for a moment no team took: the laning phase running out, or a level.
    pub(super) taken_by: Option<Team>,
}

/// Watches one game's polls for Turning points. What happened before its first poll,
/// as when the app starts mid-game, fires nothing.
#[derive(Default)]
pub(super) struct TurningPointWatch {
    seen_events: Option<usize>,
    level: Option<u32>,
    laning_over: bool,
}

impl TurningPointWatch {
    /// The latest Turning point since the last poll.
    pub(super) fn seen(&mut self, data: &AllGameData, player: &Player) -> Option<TurningPoint> {
        let now = data.game_time();
        let events = data.events();
        let Some(seen) = self.seen_events.replace(events.len()) else {
            self.level = Some(player.level);
            self.laning_over =
                now >= LANING_ENDS || events.iter().any(|event| event.name() == "TurretKilled");
            return None;
        };
        let mut latest = None;
        for event in events.get(seen..).unwrap_or_default() {
            latest = self.of_event(data, event).or(latest);
        }
        if !self.laning_over && now >= LANING_ENDS {
            self.laning_over = true;
            latest = Some(TurningPoint {
                game_time: now,
                moment: Moment::LaningTimeUp,
                taken_by: None,
            });
        }
        let last = self.level.replace(player.level).unwrap_or(player.level);
        if let Some(&level) = LEVELS
            .iter()
            .rev()
            .find(|&&level| last < level && player.level >= level)
        {
            latest = Some(TurningPoint {
                game_time: now,
                moment: Moment::Level(level),
                taken_by: None,
            });
        }
        latest
    }

    /// Later turrets, Void Grubs and kills are no Turning point.
    fn of_event(&mut self, data: &AllGameData, event: &Event) -> Option<TurningPoint> {
        let moment = match event.name() {
            "TurretKilled" if !self.laning_over => {
                self.laning_over = true;
                Moment::FirstTurret
            }
            "DragonKill" => Moment::Dragon(event.dragon_type().map(str::to_string)),
            "HeraldKill" => Moment::Herald,
            "BaronKill" => Moment::Baron,
            "InhibKilled" => Moment::Inhibitor,
            "Ace" => Moment::Ace,
            _ => return None,
        };
        Some(TurningPoint {
            game_time: event.time().unwrap_or_else(|| data.game_time()),
            moment,
            taken_by: data.taker_team(event),
        })
    }
}
