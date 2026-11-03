use std::path::Path;
use std::sync::{Arc, Mutex};

use lollipop_core::advice::{AdviceCall, Advisor, Language};
use lollipop_core::delivery::TableFile;
use lollipop_core::live::{poll, League, LiveGame, LiveUpdate};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::game_clock::GameClock;
use super::server::serve;

pub const KEY: &str = "sk-proj-Fake0123abcd";
pub const ME: &str = "Me#EUW";
const NO_TABLE: &str = "/nonexistent/benchmark-table.json";

#[derive(Clone, Copy)]
enum Stage {
    Loading,
    /// `gameTime` of the next answer, one second later on each.
    Running {
        next: f64,
    },
    Ended,
}

/// Live Client Data from the loading screen on, moved between stages by the test.
pub struct LoadingGame {
    stage: Arc<Mutex<Stage>>,
    players: Arc<Mutex<Vec<Value>>>,
    clock: GameClock,
    pub url: String,
}

impl LoadingGame {
    pub async fn at_loading_screen(players: Vec<Value>) -> Self {
        Self::serving(Stage::Loading, players).await
    }

    /// A game the app first sees after `GameStart`, as when it starts mid-game.
    pub async fn running_at(game_time: f64, players: Vec<Value>) -> Self {
        Self::serving(Stage::Running { next: game_time }, players).await
    }

    async fn serving(stage: Stage, players: Vec<Value>) -> Self {
        let stage = Arc::new(Mutex::new(stage));
        let players = Arc::new(Mutex::new(players));
        let clock = GameClock::starting_at(0.0);
        let (served_stage, served_players) = (Arc::clone(&stage), Arc::clone(&players));
        let url = serve(Box::new(move || {
            let players = served_players.lock().unwrap().clone();
            let mut stage = served_stage.lock().unwrap();
            let served = *stage;
            if let Stage::Running { next } = &mut *stage {
                *next += 1.0;
            }
            drop(stage);
            Some(body(served, &players))
        }))
        .await;
        Self {
            stage,
            players,
            clock,
            url,
        }
    }

    pub fn start_game_at(&self, game_time: f64) {
        *self.stage.lock().unwrap() = Stage::Running { next: game_time };
    }

    pub fn end_game(&self) {
        *self.stage.lock().unwrap() = Stage::Ended;
    }

    pub fn back_to_loading_screen(&self) {
        *self.stage.lock().unwrap() = Stage::Loading;
    }

    pub fn set_players(&self, players: Vec<Value>) {
        *self.players.lock().unwrap() = players;
    }

    /// Polls with Advice from `openai_url` while `on` says so.
    pub fn poll(
        &self,
        openai_url: &str,
        lockfile: &Path,
        on: Arc<Mutex<bool>>,
    ) -> mpsc::UnboundedReceiver<Option<LiveGame>> {
        let advisor = Advisor::new(openai_url, move |_| {
            (*on.lock().unwrap()).then(|| AdviceCall {
                key: KEY.to_string(),
                language: Language::Russian,
            })
        });
        let (updates, received) = mpsc::unbounded_channel();
        let (url, lockfile, clock) = (self.url.clone(), lockfile.to_path_buf(), self.clock.clone());
        tokio::spawn(async move {
            poll(
                League {
                    live_client_url: &url,
                    lockfile: &lockfile,
                },
                &TableFile::new(NO_TABLE.into()),
                &advisor,
                &clock,
                move |update: LiveUpdate| {
                    let _ = updates.send(update.game);
                },
            )
            .await;
        });
        received
    }
}

fn body(stage: Stage, players: &[Value]) -> String {
    let (events, game_time) = match stage {
        Stage::Loading => (json!([]), 0.0),
        Stage::Running { next } => (
            json!([{ "EventName": "GameStart", "EventTime": 0.0 }]),
            next,
        ),
        Stage::Ended => (
            json!([
                { "EventName": "GameStart", "EventTime": 0.0 },
                { "EventName": "GameEnd", "EventTime": 1500.0 },
            ]),
            1500.0,
        ),
    };
    json!({
        "activePlayer": {
            "riotId": ME,
            "currentGold": 0.0,
            "fullRunes": {
                "keystone": { "displayName": "Electrocute", "id": 8112 },
                "primaryRuneTree": { "displayName": "Domination", "id": 8100 },
                "secondaryRuneTree": { "displayName": "Sorcery", "id": 8200 },
                "generalRunes": [
                    { "displayName": "Electrocute", "id": 8112 },
                    { "displayName": "Taste of Blood", "id": 8139 },
                ],
                "statRunes": [{ "id": 5008, "rawDescription": "perk_tooltip_StatModAdaptive" }],
            },
        },
        "allPlayers": players,
        "events": { "Events": events },
        "gameData": { "gameTime": game_time },
    })
    .to_string()
}

pub fn player(riot_id: &str, champion: &str, team: &str, position: &str) -> Value {
    json!({
        "riotId": riot_id,
        "riotIdGameName": riot_id.split('#').next().unwrap(),
        "summonerName": riot_id,
        "championName": champion,
        "team": team,
        "position": position,
        "level": 1,
        "items": [],
        "scores": { "creepScore": 0 },
        "runes": {
            "keystone": { "displayName": "Conqueror", "id": 8010 },
            "primaryRuneTree": { "displayName": "Precision", "id": 8000 },
            "secondaryRuneTree": { "displayName": "Resolve", "id": 8400 },
        },
        "summonerSpells": {
            "summonerSpellOne": { "displayName": "Flash" },
            "summonerSpellTwo": { "displayName": "Ignite" },
        },
    })
}

/// Ten players in ranked positions, the active player in `position` as `champion`.
pub fn lineup(champion: &str, position: &str) -> Vec<Value> {
    let positions = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
    let allies = ["Garen", "Vi", "Orianna", "Jinx", "Thresh"];
    let enemies = ["Darius", "LeeSin", "Zed", "Caitlyn", "Lux"];
    let mut players = Vec::new();
    for (slot, at) in positions.iter().enumerate() {
        if *at == position {
            players.push(player(ME, champion, "ORDER", at));
        } else {
            players.push(player(
                &format!("Friend{slot}#EUW"),
                allies[slot],
                "ORDER",
                at,
            ));
        }
        players.push(player(
            &format!("Foe{slot}#EUW"),
            enemies[slot],
            "CHAOS",
            at,
        ));
    }
    players
}
