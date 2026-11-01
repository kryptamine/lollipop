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
        self.states[0]["gameTime"].as_f64().unwrap()
    }

    /// The recorder logs changes only, so the state at a time is the last one before it.
    /// `None` once the connection drops, about 1 s after `GameEnd`.
    pub fn payload_at(&self, game_time: f64) -> Option<String> {
        let last = self.states.last()?;
        if game_time > last["gameTime"].as_f64()? + 1.0 {
            return None;
        }
        let state = self
            .states
            .iter()
            .rev()
            .find(|state| state["gameTime"].as_f64().unwrap() <= game_time)?;
        let events: Vec<&Value> = self
            .events
            .iter()
            .filter(|event| event["gameTime"].as_f64().unwrap() <= game_time)
            .map(|event| &event["event"])
            .collect();
        let payload = json!({
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
        });
        Some(payload.to_string())
    }
}
