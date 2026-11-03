use serde_json::{json, Value};

use super::fake_data_dragon::{DORANS_BLADE, INFINITY_EDGE};

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
    pub minutes: u32,
    pub kills: Option<Vec<Kill>>,
}

/// When, the monster type and its sub type.
pub type Kill = (u32, &'static str, Option<&'static str>);

impl Game {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            version: String::from("16.21.824.8524"),
            duration_s: 1800,
            queue: 420,
            early_surrender: false,
            has_timeline: true,
            minutes: 2,
            kills: None,
        }
    }

    /// Holds these kills in place of the default ones, which all fall before any spawn.
    pub fn killing(id: &str, kills: &[Kill]) -> Self {
        Self {
            kills: Some(kills.to_vec()),
            ..Self::new(id)
        }
    }

    /// Past the first two frames, each frame runs 50 ms per minute late.
    pub fn lasting(id: &str, minutes: u32) -> Self {
        Self {
            minutes,
            ..Self::new(id)
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
                    "teamId": if id <= 5 { 100 } else { 200 },
                    "magicDamageDealtToChampions": 1000 * id,
                    "physicalDamageDealtToChampions": 10_000 - 1000 * id,
                    "totalDamageDealtToChampions": 10_000,
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
        let kills = self.kills.as_ref().map_or_else(default_kills, Clone::clone);
        let (first, second) = kills.split_at(kills.iter().filter(|kill| kill.0 < 60_000).count());
        let purchases = (1..=PARTICIPANTS).flat_map(|id| {
            [(30_000, DORANS_BLADE), (45_000 + id, INFINITY_EDGE)].map(|(timestamp, item)| {
                json!({
                    "type": "ITEM_PURCHASED",
                    "timestamp": timestamp,
                    "participantId": id,
                    "itemId": item,
                })
            })
        });
        let mut purchases: Vec<Value> = purchases.collect();
        purchases.sort_by_key(|event| event["timestamp"].as_u64());
        let first = [purchases, champion_kills(), elites(first)].concat();
        let mut frames = vec![
            frame(0, 0, &[json!({ "type": "PAUSE_END", "realTimestamp": 5 })]),
            frame(60_011, 1, &first),
            frame(120_050, 2, &elites(second)),
        ];
        frames.extend((3..=self.minutes).map(|index| frame(index * 60_050, index, &[])));
        json!({
            "metadata": { "matchId": self.id },
            "info": {
                "frameInterval": 60000,
                "participants": (1..=PARTICIPANTS)
                    .map(|id| json!({ "participantId": id, "puuid": self.puuid_of(id) }))
                    .collect::<Vec<_>>(),
                "frames": frames,
            },
        })
    }
}

/// 4 kills 3 at 0:20 with 2 and 5 assisting, then a tower executes 4 at 0:40.
fn champion_kills() -> Vec<Value> {
    vec![
        json!({
            "type": "CHAMPION_KILL",
            "timestamp": 20_000,
            "killerId": 4,
            "victimId": 3,
            "assistingParticipantIds": [2, 5],
            "bounty": 300,
        }),
        json!({ "type": "CHAMPION_KILL", "timestamp": 40_000, "killerId": 0, "victimId": 4 }),
    ]
}

fn default_kills() -> Vec<Kill> {
    vec![
        (50_000, "HORDE", None),
        (55_000, "HORDE", None),
        (70_000, "DRAGON", Some("FIRE_DRAGON")),
        (80_000, "DRAGON", Some("WATER_DRAGON")),
        (90_000, "DRAGON", Some("ELDER_DRAGON")),
        (100_000, "BARON_NASHOR", None),
        (110_000, "RIFTHERALD", None),
    ]
}

fn elites(kills: &[Kill]) -> Vec<Value> {
    let events = kills.iter();
    events.map(|kill| elite(kill.0, kill.1, kill.2)).collect()
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
                "championStats": { "armor": 30 + index },
                "damageStats": { "totalDamageDone": 1000 * index },
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
        "killerTeamId": 200,
        "bounty": 0,
        "position": { "x": 5007, "y": 10471 },
        "monsterType": monster_type,
    });
    if let Some(sub_type) = sub_type {
        event["monsterSubType"] = json!(sub_type);
    }
    event
}
