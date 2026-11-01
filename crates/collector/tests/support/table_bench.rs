use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rusqlite::{params, Connection};
use serde_json::Value;
use tempfile::TempDir;

/// Games of one participant, with frames exactly on the minute and steady growth.
pub struct Played {
    pub champion: &'static str,
    pub role: &'static str,
    pub tier: &'static str,
    pub patch: (u32, u32),
    pub minutes: u32,
    pub cs_per_minute: u32,
    pub gold_per_minute: u32,
    pub level: u32,
}

impl Played {
    pub const fn new(champion: &'static str, tier: &'static str) -> Self {
        Self {
            champion,
            role: "TOP",
            tier,
            patch: (16, 21),
            minutes: 25,
            cs_per_minute: 7,
            gold_per_minute: 400,
            level: 10,
        }
    }

    fn frames(&self) -> Vec<Frame> {
        (0..=self.minutes)
            .map(|minute| Frame {
                at_ms: u64::from(minute) * 60_000,
                cs: self.cs_per_minute * minute,
                total_gold: self.gold_per_minute * minute,
                level: self.level,
            })
            .collect()
    }
}

pub struct Frame {
    pub at_ms: u64,
    pub cs: u32,
    pub total_gold: u32,
    pub level: u32,
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

fn output_in(dir: &Path) -> PathBuf {
    dir.join("table.json")
}

pub fn run_build(database: &Path, reworks: &Path, dir: &Path, extra: &[&str]) -> Build {
    let output = Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
        .arg("build-table")
        .arg("--database")
        .arg(database)
        .arg("--reworks")
        .arg(reworks)
        .arg("--output")
        .arg(output_in(dir))
        .args(extra)
        .output()
        .unwrap();
    let table = std::fs::read(output_in(dir))
        .ok()
        .map(|body| serde_json::from_slice(&body).unwrap());
    Build { output, table }
}

pub struct Build {
    pub output: Output,
    pub table: Option<Value>,
}

impl Build {
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }

    pub fn table(&self) -> &Value {
        assert!(self.output.status.success(), "{}", self.stderr());
        self.table.as_ref().unwrap()
    }

    pub fn cell(&self, champion: &str, tier: &str) -> Option<&Value> {
        self.table()["cells"]
            .as_array()
            .unwrap()
            .iter()
            .find(|cell| cell["champion"] == champion && cell["tier"] == tier)
    }

    pub fn minute(&self, champion: &str, tier: &str, minute: u32) -> Option<&Value> {
        self.cell(champion, tier)?["minutes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["minute"] == minute)
    }

    /// The `n` of a minute, and 0 when the Cell or the minute is not in the file.
    pub fn n_at(&self, champion: &str, tier: &str, minute: u32) -> u64 {
        self.minute(champion, tier, minute)
            .map_or(0, |entry| entry["n"].as_u64().unwrap())
    }
}
