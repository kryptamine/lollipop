//! The `allgamedata` JSON document, one struct per nested object. Polled every second,
//! so strings borrow from the response body. `Cow` is for one with JSON escapes.

use std::borrow::Cow;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AllGameData<'a> {
    #[serde(borrow)]
    active_player: ActivePlayer<'a>,
    #[serde(borrow)]
    all_players: Vec<Player<'a>>,
    #[serde(borrow)]
    events: Events<'a>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer<'a> {
    #[serde(borrow)]
    riot_id: Option<Cow<'a, str>>,
    #[serde(borrow)]
    summoner_name: Option<Cow<'a, str>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Player<'a> {
    #[serde(borrow)]
    riot_id: Option<Cow<'a, str>>,
    #[serde(borrow)]
    summoner_name: Option<Cow<'a, str>>,
    pub(super) scores: Scores,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Scores {
    pub(super) creep_score: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Events<'a> {
    #[serde(borrow)]
    events: Vec<Event<'a>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Event<'a> {
    #[serde(borrow)]
    event_name: Cow<'a, str>,
}

impl AllGameData<'_> {
    pub(super) fn active_player(&self) -> Option<&Player<'_>> {
        let active = &self.active_player;
        self.all_players.iter().find(|p| {
            let same_riot_id = active.riot_id.is_some() && p.riot_id == active.riot_id;
            let same_name =
                active.summoner_name.is_some() && p.summoner_name == active.summoner_name;
            same_riot_id || same_name
        })
    }

    /// A 200 alone proves nothing: it can come about 56 s before `GameStart`.
    pub(super) fn match_running(&self) -> bool {
        self.has_event("GameStart") && !self.has_event("GameEnd")
    }

    fn has_event(&self, name: &str) -> bool {
        self.events.events.iter().any(|e| e.event_name == name)
    }
}
