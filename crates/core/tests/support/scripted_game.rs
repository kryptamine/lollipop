use std::sync::Mutex;

use lollipop_core::live::Snapshot;
use serde_json::{json, Value};

use super::game_clock::GameClock;
use super::polling::start_polling;
use super::server::serve;

/// The answer to one poll.
pub enum Poll {
    Served(String),
    Dropped,
}

pub const CS: u32 = 30;
pub const LEVEL: u32 = 3;

/// `items` holds an (itemID, count) per slot.
pub fn running(game_time: f64, gold: f64, items: &[(u32, u32)]) -> Poll {
    Poll::Served(payload(game_time, gold, items, &["GameStart"]))
}

pub fn loading(game_time: f64) -> Poll {
    Poll::Served(loading_body(game_time))
}

pub fn loading_body(game_time: f64) -> String {
    payload(game_time, 0.0, &[], &[])
}

pub fn ended(game_time: f64, gold: f64) -> Poll {
    Poll::Served(payload(game_time, gold, &[], &["GameStart", "GameEnd"]))
}

/// A running match as Master Yi, with Live Client Data's `position`.
pub fn as_master_yi(game_time: f64, position: &str) -> String {
    let body = json!({
        "activePlayer": { "riotId": "Me#EUW", "currentGold": 500.0 },
        "allPlayers": [{
            "riotId": "Me#EUW",
            "championName": "Master Yi",
            "position": position,
            "level": LEVEL,
            "items": [],
            "scores": { "creepScore": CS },
        }],
        "events": { "Events": [{ "EventName": "GameStart" }] },
        "gameData": { "gameTime": game_time },
    });
    body.to_string()
}

fn payload(game_time: f64, gold: f64, items: &[(u32, u32)], events: &[&str]) -> String {
    let items: Vec<Value> = items
        .iter()
        .map(|(id, count)| json!({ "itemID": id, "count": count }))
        .collect();
    let events: Vec<Value> = events.iter().map(|e| json!({ "EventName": e })).collect();
    let body = json!({
        "activePlayer": { "riotId": "Me#EUW", "currentGold": gold },
        "allPlayers": [{
            "riotId": "Me#EUW",
            "level": LEVEL,
            "items": items,
            "scores": { "creepScore": CS },
        }],
        "events": { "Events": events },
        "gameData": { "gameTime": game_time },
    });
    body.to_string()
}

/// Serves `polls` in order and returns what the poll loop made of each.
pub async fn updates_for(polls: Vec<Poll>) -> Vec<Option<Snapshot>> {
    let count = polls.len();
    let polls = Mutex::new(polls.into_iter());
    let url = serve(Box::new(move || {
        let poll = polls.lock().unwrap().next()?;
        match poll {
            Poll::Served(body) => Some(body),
            Poll::Dropped => None,
        }
    }))
    .await;
    let mut updates = start_polling(url, GameClock::starting_at(0.0));
    let mut received = Vec::with_capacity(count);
    for _ in 0..count {
        received.push(updates.recv().await.unwrap().1);
    }
    received
}
