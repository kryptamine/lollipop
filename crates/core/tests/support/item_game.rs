use std::sync::{Arc, Mutex};

use lollipop_core::advice::{AdviceCall, Advisor, Language, Trigger};
use lollipop_core::delivery::TableFile;
use lollipop_core::live::{poll, BuildItem, League, LiveUpdate, ShownAdvice};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::mpsc;

use super::fake_lcu::FakeLcu;
use super::fake_openai::FakeOpenAi;
use super::game_clock::GameClock;
use super::loading_game::{lineup, Updates, KEY, ME};
use super::server::serve;

pub const FED: &str = "Foe2#EUW";
pub const DATA_DRAGON_VERSION: &str = "16.21.1";

/// A game whose `gameTime` stands still between the moves the test makes, so an answer
/// arriving in real time lands at a known moment of the game.
struct State {
    loading: bool,
    game_time: f64,
    gold: f64,
    players: Vec<Value>,
    events: Vec<Value>,
}

/// Live Client Data and the League client for a game of Ahri in the middle lane,
/// with a Benchmark table on disk.
pub struct ItemGame {
    state: Arc<Mutex<State>>,
    lcu: FakeLcu,
    dir: TempDir,
    url: String,
}

impl ItemGame {
    /// A running game at `game_time`, its table holding `table`.
    pub async fn running_at(game_time: f64, table: &Value) -> Self {
        Self::serving(false, game_time, table).await
    }

    pub async fn at_loading_screen(table: &Value) -> Self {
        Self::serving(true, 0.0, table).await
    }

    async fn serving(loading: bool, game_time: f64, table: &Value) -> Self {
        let players = lineup("Ahri", "MIDDLE")
            .into_iter()
            .map(|mut player| {
                player["isDead"] = json!(false);
                player["respawnTimer"] = json!(0.0);
                player["scores"] =
                    json!({ "creepScore": 0, "kills": 1, "deaths": 2, "assists": 3 });
                // Zed, the usual killer, is fed so that a death to him asks for Advice.
                if player["riotId"] == FED {
                    player["scores"] =
                        json!({ "creepScore": 0, "kills": 5, "deaths": 1, "assists": 0 });
                }
                player
            })
            .collect();
        let state = Arc::new(Mutex::new(State {
            loading,
            game_time,
            gold: 1250.0,
            players,
            events: Vec::new(),
        }));
        let served = Arc::clone(&state);
        let url = serve(Box::new(move || Some(body(&served.lock().unwrap())))).await;
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("table.json"), table.to_string()).unwrap();
        Self {
            state,
            lcu: FakeLcu::start().await,
            dir,
            url,
        }
    }

    pub fn start_game(&self) {
        self.state.lock().unwrap().loading = false;
    }

    pub fn set_time(&self, game_time: f64) {
        self.state.lock().unwrap().game_time = game_time;
    }

    /// The active player killed by `killer` with `assisters`, all by Riot ID.
    pub fn die_to(&self, killer: &str, assisters: &[&str], respawn_timer: f64) {
        let at = self.state.lock().unwrap().game_time;
        self.log_event(json!({
            "EventName": "ChampionKill",
            "EventTime": at,
            "KillerName": killer,
            "VictimName": ME,
            "Assisters": assisters,
        }));
        self.set_player(ME, "isDead", json!(true));
        self.set_player(ME, "respawnTimer", json!(respawn_timer));
    }

    pub fn log_event(&self, event: Value) {
        self.state.lock().unwrap().events.push(event);
    }

    pub fn respawn(&self) {
        self.set_player(ME, "isDead", json!(false));
        self.set_player(ME, "respawnTimer", json!(0.0));
    }

    /// Items by id and the name the client shows them by.
    pub fn give_items(&self, riot_id: &str, items: &[(u32, &str)]) {
        let items: Vec<Value> = items
            .iter()
            .map(|(id, name)| json!({ "itemID": id, "count": 1, "displayName": name }))
            .collect();
        self.set_player(riot_id, "items", json!(items));
    }

    /// The active player adds `items` and pays `cost`, seen in one poll as League shows it.
    pub fn buy(&self, items: &[(u32, &str)], cost: f64) {
        let mut state = self.state.lock().unwrap();
        state.gold -= cost;
        let player = state
            .players
            .iter_mut()
            .find(|p| p["riotId"] == ME)
            .unwrap();
        let held = player["items"].as_array_mut().unwrap();
        for (id, name) in items {
            held.push(json!({ "itemID": id, "count": 1, "displayName": name }));
        }
        drop(state);
    }

    /// The active player sells every `item` they hold.
    pub fn sell(&self, item: u32) {
        let mut state = self.state.lock().unwrap();
        let player = state
            .players
            .iter_mut()
            .find(|p| p["riotId"] == ME)
            .unwrap();
        let held = player["items"].as_array_mut().unwrap();
        held.retain(|held| held["itemID"] != item);
        drop(state);
    }

    pub fn set_level(&self, level: u32) {
        self.set_player(ME, "level", json!(level));
    }

    pub fn set_gold(&self, gold: f64) {
        self.state.lock().unwrap().gold = gold;
    }

    fn set_player(&self, riot_id: &str, field: &str, value: Value) {
        let players = &mut self.state.lock().unwrap().players;
        players.iter_mut().find(|p| p["riotId"] == riot_id).unwrap()[field] = value;
    }

    /// Polls with every Trigger on and Advice in Russian.
    pub fn poll(&self, openai_url: &str) -> Updates {
        self.poll_with(openai_url, |_| true)
    }

    pub fn poll_with(
        &self,
        openai_url: &str,
        on: impl Fn(Trigger) -> bool + Send + Sync + 'static,
    ) -> Updates {
        let advisor = Advisor::new(openai_url, move |trigger| {
            on(trigger).then(|| AdviceCall {
                key: KEY.to_string(),
                language: Language::Russian,
            })
        });
        let (updates, received) = mpsc::unbounded_channel();
        self.spawn_poll(advisor, move |update| {
            let _ = updates.send(update.game);
        });
        received
    }

    /// Polls with every Trigger on and Advice in Russian, and gives every update whole.
    pub fn poll_updates(&self, openai_url: &str) -> mpsc::UnboundedReceiver<LiveUpdate> {
        let advisor = Advisor::new(openai_url, |_| {
            Some(AdviceCall {
                key: KEY.to_string(),
                language: Language::Russian,
            })
        });
        let (updates, received) = mpsc::unbounded_channel();
        self.spawn_poll(advisor, move |update| {
            let _ = updates.send(update);
        });
        received
    }

    /// Polls with no OpenAI key, and gives every update whole.
    pub fn poll_updates_without_key(&self) -> mpsc::UnboundedReceiver<LiveUpdate> {
        // Nothing listens there: with no key, no Advice is ever asked.
        let advisor = Advisor::new("http://127.0.0.1:9", |_| None);
        let (updates, received) = mpsc::unbounded_channel();
        self.spawn_poll(advisor, move |update| {
            let _ = updates.send(update);
        });
        received
    }

    /// Polls with no OpenAI key, and gives the Build of every poll.
    pub fn poll_builds(&self) -> mpsc::UnboundedReceiver<Vec<BuildItem>> {
        // Nothing listens there: with no key, no Advice is ever asked.
        let advisor = Advisor::new("http://127.0.0.1:9", |_| None);
        let (builds, received) = mpsc::unbounded_channel();
        self.spawn_poll(advisor, move |update| {
            let _ = builds.send(update.build);
        });
        received
    }

    fn spawn_poll(&self, advisor: Advisor, send: impl FnMut(LiveUpdate) + Send + 'static) {
        let url = self.url.clone();
        let lockfile = self.lcu.lockfile();
        let table = self.dir.path().join("table.json");
        tokio::spawn(async move {
            poll(
                League {
                    live_client_url: &url,
                    lockfile: &lockfile,
                },
                &TableFile::new(table),
                &advisor,
                &GameClock::starting_at(0.0),
                send,
            )
            .await;
        });
    }
}

pub const SETTLING_POLLS: usize = 8;

/// Polls until OpenAI has answered every request and the poll loop has had time to take
/// the answers, and gives the Widget then.
pub async fn settle(updates: &mut Updates, openai: &FakeOpenAi) -> Option<ShownAdvice> {
    let mut last = None;
    for _ in 0..SETTLING_POLLS {
        last = updates.recv().await.unwrap();
    }
    while openai.answered() < openai.requests().len() {
        last = updates.recv().await.unwrap();
    }
    for _ in 0..3 {
        last = updates.recv().await.unwrap();
    }
    last.and_then(|game| game.advice)
}

fn body(state: &State) -> String {
    let mut events = vec![];
    if !state.loading {
        events.push(json!({ "EventName": "GameStart", "EventTime": 0.0 }));
    }
    events.extend(state.events.iter().cloned());
    json!({
        "activePlayer": { "riotId": ME, "currentGold": state.gold },
        "allPlayers": state.players,
        "events": { "Events": events },
        "gameData": { "gameTime": if state.loading { 0.0 } else { state.game_time } },
    })
    .to_string()
}

/// A schema 2 table with no Cells and Ahri's Item candidates for the middle lane, with
/// `magic_share` per enemy champion. Against a magic-heavy team, Steelcaps, Morello and
/// the Rod lose on share or as no completed item. Only Banshee's lists its components.
pub fn item_table(magic_share: &Value) -> Value {
    let candidate = |item: u32, share: f64, minute: f64| json!({ "item": item, "share": share, "minute": minute });
    let zhonyas = candidate(3157, 0.6, 20.0);
    let mut archangels = candidate(3003, 0.4, 22.0);
    archangels["owned_as"] = json!([3040]);
    let mut banshees = candidate(3102, 0.45, 14.0);
    banshees["builds_from"] = json!([3108, 1052, 1026]);
    let name = |en: &str, ru: &str| json!({ "en": en, "ru": ru });
    json!({
        "schema_version": 2,
        "patch_window": ["16.20", "16.21"],
        "cells": [],
        "damage_profile": {
            "magic_share": magic_share,
            "mixed_from": 1.0,
            "magic_heavy_from": 1.5,
        },
        "item_candidates": [{
            "champion": "Ahri",
            "role": "MIDDLE",
            "physical_heavy": [zhonyas, candidate(3047, 0.5, 12.0)],
            "mixed": [zhonyas, candidate(3020, 0.5, 12.0)],
            "magic_heavy": [
                candidate(1058, 0.9, 8.0),
                candidate(3047, 0.3, 9.0),
                candidate(3020, 0.8, 10.0),
                zhonyas,
                banshees,
                archangels,
                candidate(3089, 0.3, 25.0),
                candidate(3135, 0.25, 28.0),
                candidate(3165, 0.1, 18.0),
            ],
        }],
        "data_dragon_version": DATA_DRAGON_VERSION,
        "item_names": {
            "3020": name("Sorcerer's Shoes", "Сапоги чародея"),
            "3047": name("Plated Steelcaps", "Бронированные сапоги"),
            "3102": name("Banshee's Veil", "Покров банши"),
        },
        "boots": [3006, 3020, 3047, 3111],
        "completed": [3003, 3089, 3102, 3135, 3157, 3165],
    })
}

/// Enemies whose magic shares sum to 1.6, the magic-heavy bucket.
pub fn magic_heavy() -> Value {
    json!({ "Darius": 0.1, "LeeSin": 0.2, "Zed": 0.1, "Caitlyn": 0.3, "Lux": 0.9 })
}

/// Enemies whose magic shares sum to 0.5, the physical-heavy bucket.
pub fn physical_heavy() -> Value {
    json!({ "Darius": 0.1, "LeeSin": 0.1, "Zed": 0.1, "Caitlyn": 0.1, "Lux": 0.1 })
}
