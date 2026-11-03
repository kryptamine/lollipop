use std::path::PathBuf;

use rusqlite::{params, Connection};
use tempfile::TempDir;

use super::archived_match::ArchivedMatch;
use super::build::{output_in, run_build, run_build_on, Build};
use super::fake_data_dragon::FakeDataDragon;
use super::frame::Frame;
use super::played::Played;

fn compressed(body: &serde_json::Value) -> Vec<u8> {
    zstd::bulk::compress(&serde_json::to_vec(body).unwrap(), 1).unwrap()
}

/// A database holding exactly the games a test names, and runs of the built binary on it.
pub struct TableBench {
    dir: TempDir,
    connection: Connection,
    next_match: u32,
}

impl TableBench {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let connection = Connection::open(dir.path().join("collector.sqlite3")).unwrap();
        connection
            .execute_batch("PRAGMA synchronous = OFF; PRAGMA journal_mode = MEMORY;")
            .unwrap();
        connection
            .execute_batch(include_str!("../../src/store/schema.sql"))
            .unwrap();
        let bench = Self {
            dir,
            connection,
            next_match: 0,
        };
        bench.set_reworks("[]");
        bench
    }

    pub fn set_reworks(&self, body: &str) {
        std::fs::write(self.reworks(), body).unwrap();
    }

    pub fn add(&mut self, count: u32, played: &Played) {
        let frames = played.frames();
        for _ in 0..count {
            self.add_game(played, &frames);
        }
    }

    pub fn add_game(&mut self, played: &Played, frames: &[Frame]) {
        self.next_match += 1;
        let id = self.next_match;
        self.connection
            .execute(
                "INSERT INTO matches (id, match_id, patch_major, patch_minor, tier)
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    id,
                    format!("EUW1_{id}"),
                    played.patch.0,
                    played.patch.1,
                    played.tier
                ],
            )
            .unwrap();
        self.connection
            .execute(
                "INSERT INTO participants (match, participant, champion, role) VALUES (?, 1, ?, ?)",
                params![id, played.champion, played.role],
            )
            .unwrap();
        for frame in frames {
            self.connection
                .execute(
                    "INSERT INTO minute_values (match, participant, at_ms, cs, total_gold, level)
                     VALUES (?, 1, ?, ?, ?, ?)",
                    params![id, frame.at_ms, frame.cs, frame.total_gold, frame.level],
                )
                .unwrap();
            if let Some([kills, deaths, assists]) = frame.scores {
                self.connection
                    .execute(
                        "INSERT INTO minute_scores (match, participant, at_ms, kills, deaths, assists)
                         VALUES (?, 1, ?, ?, ?, ?)",
                        params![id, frame.at_ms, kills, deaths, assists],
                    )
                    .unwrap();
            }
        }
    }

    /// A match with only its archived payloads: it gives Item candidates, and no Cell.
    pub fn add_archived(&mut self, count: u32, archived: &ArchivedMatch) {
        let game = compressed(&archived.game_body());
        let timeline = compressed(&archived.timeline_body());
        for _ in 0..count {
            self.next_match += 1;
            let id = self.next_match;
            self.connection
                .execute(
                    "INSERT INTO matches (id, match_id, patch_major, patch_minor, tier)
                     VALUES (?, ?, ?, ?, 'GOLD')",
                    params![id, format!("EUW1_{id}"), archived.patch.0, archived.patch.1],
                )
                .unwrap();
            self.connection
                .execute(
                    "INSERT INTO match_payloads (match, game, timeline) VALUES (?, ?, ?)",
                    params![id, game, timeline],
                )
                .unwrap();
        }
    }

    /// Stores a blob that is no zstd, as a damaged archive would hold.
    pub fn damage_archives(&self) {
        self.connection
            .execute("UPDATE match_payloads SET timeline = x'00'", [])
            .unwrap();
    }

    pub fn build(&self, extra: &[&str]) -> Build {
        run_build(&self.database(), &self.reworks(), self.dir.path(), extra)
    }

    pub fn build_on(&self, data_dragon: &FakeDataDragon, extra: &[&str]) -> Build {
        run_build_on(
            data_dragon,
            &self.database(),
            &self.reworks(),
            self.dir.path(),
            extra,
        )
    }

    pub fn database(&self) -> PathBuf {
        self.dir.path().join("collector.sqlite3")
    }

    pub fn output(&self) -> PathBuf {
        output_in(self.dir.path())
    }

    fn reworks(&self) -> PathBuf {
        self.dir.path().join("reworks.json")
    }
}
