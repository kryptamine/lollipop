//! The `allgamedata` JSON document, one struct per nested object.

use serde::Deserialize;

use super::gold_earned::ItemCounts;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AllGameData {
    active_player: ActivePlayer,
    all_players: Vec<Player>,
    events: Events,
    game_data: GameData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    current_gold: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Player {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    pub(super) champion_name: Option<String>,
    pub(super) position: Option<String>,
    pub(super) level: u32,
    items: Vec<Item>,
    pub(super) scores: Scores,
}

#[derive(Deserialize)]
struct Item {
    #[serde(rename = "itemID")]
    id: u32,
    count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Scores {
    pub(super) creep_score: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Events {
    events: Vec<Event>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Event {
    event_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameData {
    game_time: f64,
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

    pub(super) const fn current_gold(&self) -> f64 {
        self.active_player.current_gold
    }

    pub(super) const fn game_time(&self) -> f64 {
        self.game_data.game_time
    }

    /// A 200 alone proves nothing: it can come about 56 s before `GameStart`.
    pub(super) fn match_running(&self) -> bool {
        self.has_event("GameStart") && !self.has_event("GameEnd")
    }

    fn has_event(&self, name: &str) -> bool {
        self.events.events.iter().any(|e| e.event_name == name)
    }
}

impl Player {
    pub(super) fn item_counts(&self) -> ItemCounts {
        let mut counts = ItemCounts::new();
        for item in &self.items {
            *counts.entry(item.id).or_default() += item.count;
        }
        counts
    }
}
