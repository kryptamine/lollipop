//! The `allgamedata` JSON document, one struct per nested object.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AllGameData {
    active_player: ActivePlayer,
    all_players: Vec<Player>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    riot_id: Option<String>,
    summoner_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Player {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    pub(super) scores: Scores,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Scores {
    pub(super) creep_score: u32,
}

impl AllGameData {
    pub(super) fn active_player(&self) -> Option<&Player> {
        let active = &self.active_player;
        self.all_players.iter().find(|p| {
            let same_riot_id = active.riot_id.is_some() && p.riot_id == active.riot_id;
            let same_name =
                active.summoner_name.is_some() && p.summoner_name == active.summoner_name;
            same_riot_id || same_name
        })
    }
}
