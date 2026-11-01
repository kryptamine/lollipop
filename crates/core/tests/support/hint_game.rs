use std::path::{Path, PathBuf};
use std::sync::Mutex;

use lollipop_core::live::TimingHint;
use serde_json::{json, Value};

use super::fake_lcu::FakeLcu;
use super::game_clock::GameClock;
use super::polling::start_playing;
use super::server::serve;

pub const ORDER_PLAYER: &str = "Me#EUW";
pub const CHAOS_PLAYER: &str = "Rival#EUW";
const NO_TABLE: &str = "/nonexistent/benchmark-table.json";

pub fn dragon_kill(at: f64, dragon_type: &str, killer: &str) -> Value {
    json!({
        "EventName": "DragonKill",
        "EventTime": at,
        "DragonType": dragon_type,
        "KillerName": killer,
    })
}

pub fn baron_kill(at: f64) -> Value {
    json!({ "EventName": "BaronKill", "EventTime": at, "KillerName": ORDER_PLAYER })
}

/// One poll of a running game: its time and the events after `GameStart`.
pub struct HintPoll {
    pub game_time: f64,
    pub events: Vec<Value>,
}

fn body(poll: &HintPoll) -> String {
    let mut events = vec![json!({ "EventName": "GameStart", "EventTime": 0.0 })];
    events.extend(poll.events.iter().cloned());
    json!({
        "activePlayer": { "riotId": ORDER_PLAYER, "currentGold": 500.0 },
        "allPlayers": [
            player(json!({ "riotId": ORDER_PLAYER, "team": "ORDER" })),
            player(json!({ "riotId": CHAOS_PLAYER, "team": "CHAOS" })),
            player(json!({ "riotIdGameName": "Named", "team": "CHAOS" })),
            player(json!({ "summonerName": "OldName", "team": "CHAOS" })),
            player(json!({ "riotId": "Teamless#EUW" })),
            player(json!({ "riotId": "Twin#EUW", "riotIdGameName": "Twin", "team": "ORDER" })),
            player(json!({ "riotId": "Twin#KR", "riotIdGameName": "Twin", "team": "CHAOS" })),
        ],
        "events": { "Events": events },
        "gameData": { "gameTime": poll.game_time },
    })
    .to_string()
}

fn player(mut identity: Value) -> Value {
    identity["level"] = json!(1);
    identity["items"] = json!([]);
    identity["scores"] = json!({ "creepScore": 0 });
    identity
}

/// What the banner reads, top to bottom.
fn lines_of(hints: &[TimingHint]) -> Vec<String> {
    hints
        .iter()
        .map(|hint| format!("{} {}", hint.objective, hint.countdown))
        .collect()
}

/// The poll loop's banner at each poll. `before_serving` runs with each poll's index.
pub async fn banner_over(
    lockfile: &Path,
    polls: Vec<HintPoll>,
    before_serving: impl Fn(usize) + Send + 'static,
) -> Vec<Vec<String>> {
    hints_over(lockfile, polls, before_serving)
        .await
        .iter()
        .map(|hints| lines_of(hints))
        .collect()
}

/// The poll loop's hints at each poll.
pub async fn hints_over(
    lockfile: &Path,
    polls: Vec<HintPoll>,
    before_serving: impl Fn(usize) + Send + 'static,
) -> Vec<Vec<TimingHint>> {
    let count = polls.len();
    let served = Mutex::new(polls.into_iter().enumerate());
    let url = serve(Box::new(move || {
        let (index, poll) = served.lock().unwrap().next()?;
        before_serving(index);
        Some(body(&poll))
    }))
    .await;
    let clock = GameClock::starting_at(0.0);
    let mut updates = start_playing(url, lockfile, Path::new(NO_TABLE), clock);
    let mut hints = Vec::with_capacity(count);
    for _ in 0..count {
        let (_, update) = updates.recv().await.unwrap();
        // Without this, "no hints" would also pass when the whole game went missing.
        hints.push(update.unwrap().hints);
    }
    hints
}

/// The banner of a game first seen at `game_time`, with this event list.
pub async fn banner_at(lcu: &FakeLcu, game_time: f64, events: Vec<Value>) -> Vec<String> {
    banner_with_lockfile(&lcu.lockfile(), game_time, events).await
}

pub async fn banner_with_lockfile(
    lockfile: &Path,
    game_time: f64,
    events: Vec<Value>,
) -> Vec<String> {
    let polls = vec![HintPoll { game_time, events }];
    let mut banners = banner_over(lockfile, polls, |_| {}).await;
    banners.pop().unwrap()
}

/// A lockfile path nothing is at, as when the League client is not running.
pub fn no_client() -> PathBuf {
    PathBuf::from("/nonexistent/lockfile")
}
