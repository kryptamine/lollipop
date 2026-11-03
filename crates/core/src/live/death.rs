use super::payload::{AllGameData, Event, Player};

/// The active player killed by an enemy champion.
pub(super) struct Death<'a> {
    pub(super) data: &'a AllGameData,
    pub(super) player: &'a Player,
    pub(super) kill: &'a Event,
    pub(super) killer: &'a Player,
}

impl<'a> Death<'a> {
    /// `None` for a kill of someone else, and for a death to a turret, minion or monster.
    pub(super) fn of(data: &'a AllGameData, player: &'a Player, kill: &'a Event) -> Option<Self> {
        let victim = kill.victim_name.as_deref()?;
        if !player.goes_by(victim) {
            return None;
        }
        let killer = data.player_named(kill.killer_name.as_deref()?)?;
        let team = player.team()?;
        killer
            .team()
            .is_some_and(|other| other != team)
            .then_some(Self {
                data,
                player,
                kill,
                killer,
            })
    }

    /// Champions of the killer's team, the five whose damage the player faces.
    pub(super) fn enemies(&self) -> impl Iterator<Item = &'a Player> {
        let team = self.killer.team();
        self.data.players().iter().filter(move |p| p.team() == team)
    }

    /// The killer's kills of the player this game, this one included.
    pub(super) fn kills_by_killer(&self) -> usize {
        self.data
            .events_named("ChampionKill")
            .filter(|kill| kill.killer_name == self.kill.killer_name)
            .filter(|kill| {
                kill.victim_name
                    .as_deref()
                    .is_some_and(|victim| self.player.goes_by(victim))
            })
            .count()
    }

    /// The players who helped, by the names the event gives.
    pub(super) fn assisters(&self) -> impl Iterator<Item = &'a Player> {
        let data = self.data;
        self.kill
            .assisters
            .iter()
            .filter_map(move |name| data.player_named(name))
    }
}
