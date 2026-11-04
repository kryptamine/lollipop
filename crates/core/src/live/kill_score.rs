use std::fmt;

use super::event_name::EventName;
use super::payload::{AllGameData, Player};
use super::team::Team;

/// Champion kills by each team, as the scoreboard counts them: an execution by a
/// turret or minion counts for no team.
pub(super) struct KillScore {
    own: usize,
    enemy: usize,
}

impl KillScore {
    /// `None` when the player's team is unknown.
    pub(super) fn of(data: &AllGameData, player: &Player) -> Option<Self> {
        let own = player.team()?;
        let by = |team: Team| {
            data.events_named(EventName::ChampionKill)
                .filter(|kill| {
                    kill.killer_name
                        .as_deref()
                        .and_then(|killer| data.player_named(killer))
                        .and_then(Player::team)
                        == Some(team)
                })
                .count()
        };
        Some(Self {
            own: by(own),
            enemy: by(own.other()),
        })
    }
}

impl fmt::Display for KillScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player's team {}, enemy team {}", self.own, self.enemy)
    }
}
