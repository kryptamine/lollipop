use serde_json::{json, Value};

const PARTICIPANTS: u32 = 10;

/// One game on the fake Match-V5, shaped as the pilot's scrubbed samples are, with the
/// identity fields those samples lost put back.
#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub version: String,
    pub duration_s: u32,
    pub queue: u32,
    pub early_surrender: bool,
    pub has_timeline: bool,
}

impl Game {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            version: String::from("16.21.824.8524"),
            duration_s: 1800,
            queue: 420,
            early_surrender: false,
            has_timeline: true,
        }
    }

    pub fn on_patch(id: &str, version: &str) -> Self {
        Self {
            version: version.to_string(),
            ..Self::new(id)
        }
    }

    pub fn puuid_of(&self, participant: u32) -> String {
        format!("puuid-of-{}-{participant}", self.id)
    }

    pub fn match_body(&self) -> Value {
        let participants: Vec<Value> = (1..=PARTICIPANTS)
            .map(|id| {
                json!({
                    "participantId": id,
                    "puuid": self.puuid_of(id),
                    "riotIdGameName": format!("name-of-{}-{id}", self.id),
                    "championName": format!("Champion{id}"),
                    "teamPosition": if id == 5 { "UTILITY" } else { "TOP" },
                    "gameEndedInEarlySurrender": self.early_surrender && id == 7,
                    "goldEarned": 12000,
                })
            })
            .collect();
        json!({
            "metadata": {
                "matchId": self.id,
                "participants": (1..=PARTICIPANTS).map(|id| self.puuid_of(id)).collect::<Vec<_>>(),
            },
            "info": {
                "endOfGameResult": "GameComplete",
                "gameDuration": self.duration_s,
                "gameVersion": self.version,
                "queueId": self.queue,
                "participants": participants,
            },
        })
    }

    pub fn timeline_body(&self) -> Value {
        json!({
            "metadata": { "matchId": self.id },
            "info": {
                "frameInterval": 60000,
                "participants": (1..=PARTICIPANTS)
                    .map(|id| json!({ "participantId": id, "puuid": self.puuid_of(id) }))
                    .collect::<Vec<_>>(),
                "frames": [
                    frame(0, 0, &[json!({ "type": "PAUSE_END", "realTimestamp": 5 })]),
                    frame(60_011, 1, &[
                        json!({ "type": "ITEM_PURCHASED", "timestamp": 30_000, "itemId": 1055 }),
                        elite(50_000, "HORDE", None),
                        elite(55_000, "HORDE", None),
                    ]),
                    frame(120_050, 2, &[
                        elite(70_000, "DRAGON", Some("FIRE_DRAGON")),
                        elite(80_000, "DRAGON", Some("WATER_DRAGON")),
                        elite(90_000, "DRAGON", Some("ELDER_DRAGON")),
                        elite(100_000, "BARON_NASHOR", None),
                        elite(110_000, "RIFTHERALD", None),
                    ]),
                ],
            },
        })
    }
}

/// CS and gold follow the participant and the frame, so a mixed-up row shows.
fn frame(timestamp: u32, index: u32, events: &[Value]) -> Value {
    let frames: serde_json::Map<String, Value> = (1..=PARTICIPANTS)
        .map(|id| {
            let body = json!({
                "participantId": id,
                "minionsKilled": 10 * id * index,
                "jungleMinionsKilled": id,
                "totalGold": 500 + 100 * id * index,
                "currentGold": 1,
                "level": 1 + index,
            });
            (id.to_string(), body)
        })
        .collect();
    json!({ "timestamp": timestamp, "participantFrames": frames, "events": events })
}

fn elite(timestamp: u32, monster_type: &str, sub_type: Option<&str>) -> Value {
    let mut event = json!({
        "type": "ELITE_MONSTER_KILL",
        "timestamp": timestamp,
        "killerId": 7,
        "monsterType": monster_type,
    });
    if let Some(sub_type) = sub_type {
        event["monsterSubType"] = json!(sub_type);
    }
    event
}
