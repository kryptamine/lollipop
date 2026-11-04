use std::fmt;

use super::event_name::EventName;
use super::payload::AllGameData;
use super::team::Team;

/// What one team has taken this game, read from the whole event list.
pub(super) struct ObjectiveScore {
    /// By `DragonType`, the Elder included, oldest first.
    dragons: Vec<String>,
    heralds: usize,
    barons: usize,
    turrets: usize,
    inhibitors: usize,
}

impl ObjectiveScore {
    pub(super) fn of(data: &AllGameData, team: Team) -> Self {
        let taken = |name| {
            data.events_named(name)
                .filter(move |event| data.taker_team(event) == Some(team))
        };
        Self {
            dragons: taken(EventName::DragonKill)
                .map(|kill| kill.dragon_type().unwrap_or("unknown").to_string())
                .collect(),
            heralds: taken(EventName::HeraldKill).count(),
            barons: taken(EventName::BaronKill).count(),
            turrets: taken(EventName::TurretKilled).count(),
            inhibitors: taken(EventName::InhibKilled).count(),
        }
    }
}

impl fmt::Display for ObjectiveScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dragons = if self.dragons.is_empty() {
            "none".to_string()
        } else {
            self.dragons.join(", ")
        };
        write!(
            f,
            "dragons {dragons}; Herald {}; Baron {}; turrets {}; inhibitors {}",
            self.heralds, self.barons, self.turrets, self.inhibitors
        )
    }
}
