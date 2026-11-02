use serde_json::{Map, Value};

// Every key that names a player. PUUIDs are never stored, and the rest leads back to one.
const IDENTITY_KEYS: [&str; 7] = [
    "puuid",
    "summonerId",
    "summonerName",
    "riotIdGameName",
    "riotIdName",
    "riotIdTagline",
    "profileIcon",
];
// Per-minute champion and damage stats are most of a timeline's size.
const FRAME_STAT_KEYS: [&str; 2] = ["championStats", "damageStats"];
const ZSTD_LEVEL: i32 = 19;

/// A match's Match-V5 bodies as zstd JSON, kept for statistics no table reads yet.
pub(crate) struct ArchivedPayloads {
    pub(crate) game: Vec<u8>,
    pub(crate) timeline: Vec<u8>,
}

impl ArchivedPayloads {
    pub(crate) fn new(mut game: Value, mut timeline: Value) -> std::io::Result<Self> {
        forget_players(&mut game);
        forget_players(&mut timeline);
        drop_frame_stats(&mut timeline);
        Ok(Self {
            game: compressed(&game)?,
            timeline: compressed(&timeline)?,
        })
    }
}

/// `metadata.participants` is a bare list of PUUIDs, which no key name gives away.
fn forget_players(body: &mut Value) {
    if let Some(metadata) = body.get_mut("metadata").and_then(Value::as_object_mut) {
        metadata.remove("participants");
    }
    remove_keys(body, &IDENTITY_KEYS);
}

fn drop_frame_stats(timeline: &mut Value) {
    let frames = timeline
        .pointer_mut("/info/frames")
        .and_then(Value::as_array_mut);
    for frame in frames.into_iter().flatten() {
        let participants = frame
            .get_mut("participantFrames")
            .and_then(Value::as_object_mut);
        for participant in participants.into_iter().flat_map(Map::values_mut) {
            remove_keys(participant, &FRAME_STAT_KEYS);
        }
    }
}

fn remove_keys(value: &mut Value, keys: &[&str]) {
    match value {
        Value::Object(object) => {
            object.retain(|key, _| !keys.contains(&key.as_str()));
            object
                .values_mut()
                .for_each(|child| remove_keys(child, keys));
        }
        Value::Array(items) => items.iter_mut().for_each(|item| remove_keys(item, keys)),
        _ => {}
    }
}

fn compressed(body: &Value) -> std::io::Result<Vec<u8>> {
    zstd::bulk::compress(&serde_json::to_vec(body)?, ZSTD_LEVEL)
}
