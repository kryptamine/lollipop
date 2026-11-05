use std::path::PathBuf;
use std::sync::Mutex;

use lollipop_core::live::{LiveGame, Markers, ShownBenchmarks};
use serde_json::{json, Value};
use tempfile::TempDir;

use super::benchmark_file::{cell, flat_minutes, table_of};
use super::fake_lcu::FakeLcu;
use super::game_clock::GameClock;
use super::polling::start_playing;
use super::server::serve;

pub const CHAMPION: &str = "MasterYi";
pub const ROLE: &str = "MIDDLE";
pub const TIER: &str = "GOLD";

/// One poll of a running game. Serves Master Yi at mid unless a test says otherwise.
#[derive(Clone, Copy)]
pub struct Moment {
    pub game_time: f64,
    pub cs: u32,
    pub gold: f64,
    pub level: u32,
    /// Kills, deaths and assists, left out of the payload when `None`.
    pub scores: Option<[u32; 3]>,
}

impl Moment {
    pub const fn at(game_time: f64) -> Self {
        Self {
            game_time,
            cs: 95,
            gold: 500.0,
            level: 10,
            scores: None,
        }
    }

    pub const fn cs(self, cs: u32) -> Self {
        Self { cs, ..self }
    }

    pub const fn gold(self, gold: f64) -> Self {
        Self { gold, ..self }
    }

    pub const fn level(self, level: u32) -> Self {
        Self { level, ..self }
    }

    pub const fn scores(self, kills: u32, deaths: u32, assists: u32) -> Self {
        Self {
            scores: Some([kills, deaths, assists]),
            ..self
        }
    }
}

/// The names and role Live Client Data gives the active player.
#[derive(Clone, Copy)]
pub struct Seat {
    pub champion: &'static str,
    pub raw_champion: Option<&'static str>,
    pub position: &'static str,
}

pub const MASTER_YI_MID: Seat = Seat {
    champion: "Master Yi",
    raw_champion: None,
    position: ROLE,
};

pub fn body(seat: Seat, moment: Moment, events: &[&str]) -> String {
    let events: Vec<Value> = events.iter().map(|e| json!({ "EventName": e })).collect();
    let mut player = json!({
        "riotId": "Me#EUW",
        "championName": seat.champion,
        "position": seat.position,
        "level": moment.level,
        "items": [],
        "scores": { "creepScore": moment.cs },
    });
    if let Some([kills, deaths, assists]) = moment.scores {
        player["scores"]["kills"] = json!(kills);
        player["scores"]["deaths"] = json!(deaths);
        player["scores"]["assists"] = json!(assists);
    }
    if let Some(raw) = seat.raw_champion {
        player["rawChampionName"] = json!(raw);
    }
    json!({
        "activePlayer": { "riotId": "Me#EUW", "currentGold": moment.gold },
        "allPlayers": [player],
        "events": { "Events": events },
        "gameData": { "gameTime": moment.game_time },
    })
    .to_string()
}

/// One poll: a moment of a running game, or the `GameEnd` after one.
#[derive(Clone, Copy)]
pub enum Beat {
    Running(Moment),
    Ended(Moment),
}

/// The League client, a table on disk, and a game served through the real poll loop.
pub struct MarkerBench {
    pub lcu: FakeLcu,
    dir: TempDir,
    pub seat: Seat,
}

impl MarkerBench {
    /// In queue 420 on patch 16.21 at Gold, with no table written yet.
    pub async fn new() -> Self {
        Self {
            lcu: FakeLcu::start().await,
            dir: TempDir::new().unwrap(),
            seat: MASTER_YI_MID,
        }
    }

    /// With one Cell for the seat, flat at these Benchmarks.
    pub async fn with_flat_cell(cs: f64, gold: f64, level: f64) -> Self {
        let bench = Self::new().await;
        bench.write_table(&table_of(&[cell(
            CHAMPION,
            ROLE,
            TIER,
            &flat_minutes(cs, gold, level),
        )]));
        bench
    }

    pub fn table(&self) -> PathBuf {
        self.dir.path().join("benchmark-table.json")
    }

    pub fn write_table(&self, body: &str) {
        std::fs::write(self.table(), body).unwrap();
    }

    /// What the poll loop made of each moment.
    pub async fn markers_for(&self, moments: &[Moment]) -> Vec<Markers> {
        let beats = moments.iter().copied().map(Beat::Running).collect();
        self.markers_over(beats, |_| {}).await
    }

    /// `before_serving` runs with each poll's index, just before that poll is answered.
    pub async fn markers_over(
        &self,
        beats: Vec<Beat>,
        before_serving: impl Fn(usize) + Send + 'static,
    ) -> Vec<Markers> {
        self.over(beats, before_serving, |game| game.markers).await
    }

    /// The Benchmarks shown beside the Markers at each moment.
    pub async fn benchmarks_for(&self, moments: &[Moment]) -> Vec<ShownBenchmarks> {
        let beats = moments.iter().copied().map(Beat::Running).collect();
        self.over(beats, |_| {}, |game| game.benchmarks).await
    }

    /// The CS pace the strip shows at each moment, with or without a table.
    pub async fn paces_for(&self, moments: &[Moment]) -> Vec<Option<f64>> {
        let beats = moments.iter().copied().map(Beat::Running).collect();
        self.over(
            beats,
            |_| {},
            |game| game.snapshot.cs_pace.map(|pace| pace.per_minute),
        )
        .await
    }

    async fn over<T: Default>(
        &self,
        beats: Vec<Beat>,
        before_serving: impl Fn(usize) + Send + 'static,
        read: impl Fn(&LiveGame) -> T,
    ) -> Vec<T> {
        let seat = self.seat;
        let in_order = beats.clone();
        let served = Mutex::new(beats.into_iter().enumerate());
        let url = serve(Box::new(move || {
            let (index, beat) = served.lock().unwrap().next()?;
            before_serving(index);
            Some(match beat {
                Beat::Running(moment) => body(seat, moment, &["GameStart"]),
                Beat::Ended(moment) => body(seat, moment, &["GameStart", "GameEnd"]),
            })
        }))
        .await;
        let clock = GameClock::starting_at(0.0);
        let mut updates = start_playing(url, &self.lcu.lockfile(), &self.table(), clock);
        let mut received = Vec::with_capacity(in_order.len());
        for beat in in_order {
            let (_, update) = updates.recv().await.unwrap();
            // Without this, "no Markers" would also pass when the whole game went missing,
            // and the spec keeps the values on screen when there is nothing to compare.
            assert_eq!(update.is_some(), matches!(beat, Beat::Running(_)));
            received.push(update.as_ref().map_or_else(T::default, &read));
        }
        received
    }
}
