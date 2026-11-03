use serde_json::{json, Value};

/// A champion that deals only physical damage, filling the seats a test does not name.
pub const FILLER: &str = "Filler";

pub struct Seat {
    pub champion: &'static str,
    pub role: &'static str,
    pub magic: u64,
    pub total: u64,
}

/// One match as the collector archives it, with ten seats: 1 to 5 on team 100, the
/// rest on 200. Only the fields the table build reads, and some it must skip.
pub struct ArchivedMatch {
    pub patch: (u32, u32),
    seats: Vec<Seat>,
    events: Vec<Value>,
}

impl ArchivedMatch {
    pub fn new() -> Self {
        let seats = (0..10)
            .map(|_| Seat {
                champion: FILLER,
                role: "TOP",
                magic: 0,
                total: 10_000,
            })
            .collect();
        Self {
            patch: (16, 21),
            seats,
            // The support item granted at start sits at participant 0, which is no player.
            events: vec![purchase(0, 0, 3865), purchase(0, 0, 3865)],
        }
    }

    pub const fn on_patch(mut self, major: u32, minor: u32) -> Self {
        self.patch = (major, minor);
        self
    }

    pub fn seat(mut self, participant: u32, seat: Seat) -> Self {
        self.seats[usize::try_from(participant - 1).unwrap()] = seat;
        self
    }

    /// Seats a team of five, all dealing `magic` of 10,000 damage as magic.
    pub fn team(mut self, team: u32, champion: &'static str, magic: u64) -> Self {
        let first = if team == 100 { 1 } else { 6 };
        for participant in first..first + 5 {
            self = self.seat(participant, damage(champion, magic));
        }
        self
    }

    pub fn buys(mut self, participant: u32, item: u32, minute: u64) -> Self {
        let at_ms = minute * 60_000;
        self.events.push(purchase(participant, at_ms, item));
        self
    }

    /// An undo in the shop: `after` 0 takes back a purchase of `before`.
    pub fn undoes(mut self, participant: u32, before: u32, after: u32, minute: u64) -> Self {
        self.events.push(json!({
            "type": "ITEM_UNDO",
            "timestamp": minute * 60_000,
            "participantId": participant,
            "beforeId": before,
            "afterId": after,
            "goldGain": if after == 0 { 300 } else { -300 },
        }));
        self
    }

    pub fn game_body(&self) -> Value {
        let participants: Vec<Value> = self
            .seats
            .iter()
            .zip(1..)
            .map(|(seat, id): (&Seat, u32)| {
                json!({
                    "participantId": id,
                    "championName": seat.champion,
                    "teamPosition": seat.role,
                    "teamId": if id <= 5 { 100 } else { 200 },
                    "magicDamageDealtToChampions": seat.magic,
                    "physicalDamageDealtToChampions": seat.total - seat.magic,
                    "totalDamageDealtToChampions": seat.total,
                    "item0": 0,
                })
            })
            .collect();
        json!({ "info": { "gameVersion": "16.21.1.1", "participants": participants } })
    }

    pub fn timeline_body(&self) -> Value {
        let mut events = self.events.clone();
        events.sort_by_key(|event| event["timestamp"].as_u64().unwrap());
        events.push(json!({ "type": "ITEM_SOLD", "timestamp": 2_000_000, "participantId": 1, "itemId": 3031 }));
        let frames = vec![
            json!({ "timestamp": 0, "participantFrames": {}, "events": [] }),
            json!({ "timestamp": 60_000, "participantFrames": {}, "events": events }),
        ];
        json!({ "info": { "frameInterval": 60000, "frames": frames } })
    }
}

/// Of 10,000 damage dealt to champions, `magic` is magic.
pub const fn damage(champion: &'static str, magic: u64) -> Seat {
    Seat {
        champion,
        role: "TOP",
        magic,
        total: 10_000,
    }
}

fn purchase(participant: u32, at_ms: u64, item: u32) -> Value {
    json!({ "type": "ITEM_PURCHASED", "timestamp": at_ms, "participantId": participant, "itemId": item })
}
