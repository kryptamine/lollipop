//! The `allgamedata` JSON document, one struct per nested object.

use serde::Deserialize;

use super::gold_earned::ItemCounts;
use super::team::Team;

const RAW_CHAMPION_PREFIX: &str = "game_character_displayname_";

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
    riot_id_game_name: Option<String>,
    summoner_name: Option<String>,
    team: Option<String>,
    pub(super) champion_name: Option<String>,
    raw_champion_name: Option<String>,
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
pub(super) struct Event {
    #[serde(rename = "EventName")]
    name: String,
    #[serde(rename = "EventTime")]
    time: Option<f64>,
    killer_name: Option<String>,
    dragon_type: Option<String>,
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
        self.events.events.iter().any(|e| e.name == name)
    }

    /// The whole game's events of one kind, oldest first.
    pub(super) fn events_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Event> {
        self.events.events.iter().filter(move |e| e.name == name)
    }

    pub(super) fn killer_team(&self, kill: &Event) -> Option<Team> {
        let killer = kill.killer_name.as_deref()?;
        self.all_players
            .iter()
            .find(|p| p.goes_by(killer))
            .map_or_else(
                || Team::of_minion(killer),
                |player| player.team.as_deref().and_then(Team::of_player),
            )
    }
}

impl Event {
    /// `EventTime`, not the poll that saw it: an event can be logged seconds late.
    pub(super) const fn time(&self) -> Option<f64> {
        self.time
    }

    pub(super) fn is_elder(&self) -> bool {
        self.dragon_type.as_deref() == Some("Elder")
    }
}

impl Player {
    // Events give "a player name", and no recording shows which of these it is.
    fn goes_by(&self, name: &str) -> bool {
        [&self.riot_id, &self.riot_id_game_name, &self.summoner_name]
            .into_iter()
            .any(|known| known.as_deref() == Some(name))
    }

    /// The name a Benchmark Cell goes by. `rawChampionName` carries Match-V5's id and,
    /// unlike `championName`, should not follow the client's language.
    pub(super) fn cell_champion(&self) -> Option<&str> {
        let raw = self
            .raw_champion_name
            .as_deref()
            .and_then(|raw| raw.strip_prefix(RAW_CHAMPION_PREFIX));
        raw.or(self.champion_name.as_deref())
    }

    pub(super) fn item_counts(&self) -> ItemCounts {
        let mut counts = ItemCounts::new();
        for item in &self.items {
            *counts.entry(item.id).or_default() += item.count;
        }
        counts
    }
}
