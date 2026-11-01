use std::path::PathBuf;

use rusqlite::{params, Connection};
use tempfile::TempDir;

use super::build::{output_in, run_build, Build};
use super::frame::Frame;
use super::played::Played;

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
        }
    }

    pub fn build(&self, extra: &[&str]) -> Build {
        run_build(&self.database(), &self.reworks(), self.dir.path(), extra)
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
