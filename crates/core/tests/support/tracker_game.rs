use std::path::Path;
use std::sync::Mutex;

use lollipop_core::live::EnemyTracker;
use serde_json::{json, Value};

use super::game_clock::GameClock;
use super::polling::start_playing;
use super::server::serve;

const NO_TABLE: &str = "/nonexistent/benchmark-table.json";

pub const ME: &str = "Me#EUW";
pub const ALLY_BOT: &str = "Jinx#EUW";
pub const ENEMY_TOP: &str = "Darius#EUW";
pub const ENEMY_JUNGLER: &str = "Vi#EUW";
pub const ENEMY_MID: &str = "Zed#EUW";
pub const ENEMY_BOT: &str = "Caitlyn#EUW";

// The player is mid, so Zed is their Lane opponent.
const PLAYERS: [(&str, &str, &str, &str); 10] = [
    (ME, "Ahri", "ORDER", "MIDDLE"),
    ("Garen#EUW", "Garen", "ORDER", "TOP"),
    ("LeeSin#EUW", "Lee Sin", "ORDER", "JUNGLE"),
    (ALLY_BOT, "Jinx", "ORDER", "BOTTOM"),
    ("Lulu#EUW", "Lulu", "ORDER", "UTILITY"),
    (ENEMY_TOP, "Darius", "CHAOS", "TOP"),
    (ENEMY_JUNGLER, "Vi", "CHAOS", "JUNGLE"),
    (ENEMY_MID, "Zed", "CHAOS", "MIDDLE"),
    (ENEMY_BOT, "Caitlyn", "CHAOS", "BOTTOM"),
    ("Thresh#EUW", "Thresh", "CHAOS", "UTILITY"),
];

pub const LONG_SWORD: u32 = 1036;
pub const SERRATED_DIRK: u32 = 3134;
pub const HEALTH_POTION: u32 = 2003;
pub const MANAMUNE: u32 = 3004;
pub const MURAMANA: u32 = 3042;

const fn item_name(id: u32) -> &'static str {
    match id {
        LONG_SWORD => "Long Sword",
        SERRATED_DIRK => "Serrated Dirk",
        HEALTH_POTION => "Health Potion",
        MANAMUNE => "Manamune",
        MURAMANA => "Muramana",
        _ => "Other",
    }
}

/// One poll of a running ten-player game. Everyone is alive and holds nothing unless
/// the poll says otherwise.
pub struct TrackerPoll {
    game_time: f64,
    events: Vec<Value>,
    dead: Vec<(&'static str, f64)>,
    items: Vec<(&'static str, Vec<u32>)>,
}

impl TrackerPoll {
    pub const fn at(game_time: f64) -> Self {
        Self {
            game_time,
            events: Vec::new(),
            dead: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn dead(mut self, riot_id: &'static str, respawn_in: f64) -> Self {
        self.dead.push((riot_id, respawn_in));
        self
    }

    pub fn holding(mut self, riot_id: &'static str, items: &[u32]) -> Self {
        self.items.push((riot_id, items.to_vec()));
        self
    }

    pub fn event(mut self, event: Value) -> Self {
        self.events.push(event);
        self
    }

    fn body(&self) -> String {
        let mut events = vec![json!({ "EventName": "GameStart", "EventTime": 0.0 })];
        events.extend(self.events.iter().cloned());
        let players: Vec<Value> = PLAYERS
            .iter()
            .map(|(riot_id, champion, team, position)| {
                self.player(riot_id, champion, team, position)
            })
            .collect();
        json!({
            "activePlayer": { "riotId": ME, "currentGold": 500.0 },
            "allPlayers": players,
            "events": { "Events": events },
            "gameData": { "gameTime": self.game_time },
        })
        .to_string()
    }

    fn player(&self, riot_id: &str, champion: &str, team: &str, position: &str) -> Value {
        let respawn = self.dead.iter().find(|(dead, _)| *dead == riot_id);
        let items: Vec<Value> = self
            .items
            .iter()
            .find(|(holder, _)| *holder == riot_id)
            .map(|(_, items)| items.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|id| json!({ "itemID": id, "count": 1, "displayName": item_name(*id) }))
            .collect();
        json!({
            "riotId": riot_id,
            "championName": champion,
            "team": team,
            "position": position,
            "level": 6,
            "isDead": respawn.is_some(),
            "respawnTimer": respawn.map_or(0.0, |(_, left)| *left),
            "items": items,
            "scores": { "creepScore": 50 },
        })
    }
}

pub fn champion_kill(at: f64, killer: &str, victim: &str, assisters: &[&str]) -> Value {
    json!({
        "EventName": "ChampionKill",
        "EventTime": at,
        "KillerName": killer,
        "VictimName": victim,
        "Assisters": assisters,
    })
}

pub fn objective_kill(name: &str, at: f64, killer: &str) -> Value {
    json!({
        "EventName": name,
        "EventTime": at,
        "DragonType": "Air",
        "KillerName": killer,
        "Assisters": [],
    })
}

/// The Enemy tracker at each poll.
pub async fn trackers_over(lockfile: &Path, polls: Vec<TrackerPoll>) -> Vec<EnemyTracker> {
    let count = polls.len();
    let served = Mutex::new(polls.into_iter());
    let url = serve(Box::new(move || {
        Some(served.lock().unwrap().next()?.body())
    }))
    .await;
    let mut updates = start_playing(
        url,
        lockfile,
        Path::new(NO_TABLE),
        GameClock::starting_at(0.0),
    );
    let mut trackers = Vec::with_capacity(count);
    for _ in 0..count {
        let (_, update) = updates.recv().await.unwrap();
        trackers.push(update.unwrap().tracker);
    }
    trackers
}
