use std::path::Path;

use serde_json::{json, Value};

/// The Summoner's Rift game in the recording; the two before it are Practice Tool.
const RECORDED_GAME_START: f64 = 2724.0;

pub struct RecordedGame {
    states: Vec<Value>,
    events: Vec<Value>,
}

impl RecordedGame {
    pub fn load() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/data/live-game-probe/recorder.ndjson");
        let (states, events) = std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|record| record["t"].as_f64().unwrap() >= RECORDED_GAME_START)
            .filter(|record| record["kind"] == "state" || record["kind"] == "event")
            .partition(|record| record["kind"] == "state");
        Self { states, events }
    }

    pub fn first_game_time(&self) -> f64 {
        game_time(&self.states[0])
    }

    /// `None` once the connection drops, about 1 s after `GameEnd`.
    pub fn payload_at(&self, time: f64) -> Option<String> {
        if time > game_time(self.states.last()?) + 1.0 {
            return None;
        }
        let state = self.state_at(time)?;
        Some(payload(state, &self.events_until(time)).to_string())
    }

    /// The recorder logs changes only, so the state at a time is the last one before it.
    fn state_at(&self, time: f64) -> Option<&Value> {
        self.states.iter().rev().find(|s| game_time(s) <= time)
    }

    fn events_until(&self, time: f64) -> Vec<&Value> {
        self.events
            .iter()
            .filter(|event| game_time(event) <= time)
            .map(|event| &event["event"])
            .collect()
    }
}

fn game_time(record: &Value) -> f64 {
    record["gameTime"].as_f64().unwrap()
}

fn payload(state: &Value, events: &[&Value]) -> Value {
    json!({
        "activePlayer": { "riotId": "self", "level": state["level"], "currentGold": state["gold"] },
        "allPlayers": [{
            "riotId": "self",
            "championName": state["champion"],
            "position": state["position"],
            "level": state["level"],
            "items": state["items"],
            "scores": { "creepScore": state["cs"] },
        }],
        "events": { "Events": events },
        "gameData": { "gameTime": state["gameTime"], "gameMode": state["gameMode"] },
    })
}
