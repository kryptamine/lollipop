use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

use super::fake_data_dragon::FakeDataDragon;

pub fn output_in(dir: &Path) -> PathBuf {
    dir.join("table.json")
}

pub fn run_build(database: &Path, reworks: &Path, dir: &Path, extra: &[&str]) -> Build {
    run_build_on(&FakeDataDragon::start(), database, reworks, dir, extra)
}

pub fn run_build_on(
    data_dragon: &FakeDataDragon,
    database: &Path,
    reworks: &Path,
    dir: &Path,
    extra: &[&str],
) -> Build {
    let output = Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
        .arg("build-table")
        .arg("--database")
        .arg(database)
        .arg("--reworks")
        .arg(reworks)
        .arg("--output")
        .arg(output_in(dir))
        .args(["--data-dragon-url", &data_dragon.url])
        .args(extra)
        .output()
        .unwrap();
    let table = std::fs::read(output_in(dir))
        .ok()
        .map(|body| serde_json::from_slice(&body).unwrap());
    Build { output, table }
}

pub type Candidate = (u32, f64, f64);

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

    /// The candidates for a champion and role facing one Damage profile bucket, as
    /// `(item, share, minute)`; `None` when the file has no entry for them.
    pub fn candidates(&self, champion: &str, role: &str, bucket: &str) -> Option<Vec<Candidate>> {
        let entry = self.table()["item_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["champion"] == champion && entry["role"] == role)?;
        let list = entry[bucket].as_array().unwrap().iter().map(|candidate| {
            (
                u32::try_from(candidate["item"].as_u64().unwrap()).unwrap(),
                candidate["share"].as_f64().unwrap(),
                candidate["minute"].as_f64().unwrap(),
            )
        });
        Some(list.collect())
    }

    /// The `n` of a minute, and 0 when the Cell or the minute is not in the file.
    pub fn n_at(&self, champion: &str, tier: &str, minute: u32) -> u64 {
        self.minute(champion, tier, minute)
            .map_or(0, |entry| entry["n"].as_u64().unwrap())
    }
}
