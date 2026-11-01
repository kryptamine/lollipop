use std::ffi::{OsStr, OsString};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};
use tempfile::TempDir;

/// The built binary against a fake `gh` first on its `PATH`, with the Release in a directory.
pub struct ReleaseBench {
    dir: TempDir,
    next_file: u32,
}

impl ReleaseBench {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("bin")).unwrap();
        let gh = dir.path().join("bin/gh");
        std::fs::write(&gh, include_str!("fake_gh.sh")).unwrap();
        std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let bench = Self { dir, next_file: 0 };
        bench.set_reworks("[]");
        bench
    }

    pub fn set_reworks(&self, body: &str) {
        std::fs::write(self.reworks(), body).unwrap();
    }

    pub fn reworks(&self) -> PathBuf {
        self.dir.path().join("reworks.json")
    }

    /// Runs of this `gh` command fail until `heal` is called.
    pub fn break_gh(&self, command: &str) {
        std::fs::write(self.dir.path().join(format!("fail-{command}")), "").unwrap();
    }

    /// An upload of this asset fails after the old copy is gone, as `--clobber` can.
    pub fn lose_upload(&self, asset: &str) {
        std::fs::write(self.dir.path().join(format!("lose-upload-{asset}")), "").unwrap();
    }

    pub fn heal(&self) {
        for entry in std::fs::read_dir(self.dir.path()).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with("fail-") || name.starts_with("lose-upload-") {
                std::fs::remove_file(path).unwrap();
            }
        }
    }

    pub fn publish_file(&mut self, body: &str) -> Output {
        self.next_file += 1;
        let file = self
            .dir
            .path()
            .join(format!("built-{}.json", self.next_file));
        std::fs::write(&file, body).unwrap();
        let table = [OsStr::new("--table"), file.as_os_str()];
        self.run_publish(&table.map(OsString::from))
    }

    pub fn publish_table(&mut self, table: &Value) -> Output {
        self.publish_file(&table.to_string())
    }

    pub fn publish_built(&self, database: &Path, extra: &[&str]) -> Output {
        let mut args = vec![OsString::from("--database"), database.into()];
        args.extend(extra.iter().map(OsString::from));
        self.run_publish(&args)
    }

    fn run_publish(&self, args: &[OsString]) -> Output {
        self.run(
            "publish",
            &[&[OsString::from("--reworks"), self.reworks().into()], args].concat(),
        )
    }

    pub fn rollback(&self, version: u32) -> Output {
        let args = [OsString::from("--version"), version.to_string().into()];
        self.run("rollback", &args)
    }

    fn run(&self, command: &str, args: &[OsString]) -> Output {
        let mut path = self.dir.path().join("bin").into_os_string();
        path.push(":");
        path.push(std::env::var_os("PATH").unwrap());
        Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
            .arg(command)
            .args(args)
            .arg("--work-dir")
            .arg(self.dir.path().join("work"))
            .env("PATH", path)
            .env("FAKE_GH_STATE", self.dir.path())
            .output()
            .unwrap()
    }

    pub fn tag(&self) -> String {
        let tag = std::fs::read_to_string(self.dir.path().join("tag")).unwrap();
        tag.trim().to_string()
    }

    pub fn asset_names(&self) -> Vec<String> {
        let Ok(assets) = std::fs::read_dir(self.dir.path().join("assets")) else {
            return Vec::new();
        };
        let mut names: Vec<String> = assets
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    pub fn asset_path(&self, name: &str) -> PathBuf {
        self.dir.path().join("assets").join(name)
    }

    pub fn asset(&self, name: &str) -> Value {
        serde_json::from_slice(&std::fs::read(self.asset_path(name)).unwrap()).unwrap()
    }

    pub fn manifest_version(&self) -> u64 {
        self.asset("manifest.json")["version"].as_u64().unwrap()
    }

    pub fn issues(&self) -> Vec<String> {
        std::fs::read_to_string(self.dir.path().join("issues"))
            .map(|issues| issues.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// Corrupts a kept asset in place, as a bad upload or a hand edit would.
    pub fn overwrite_asset(&self, name: &str, body: &str) {
        std::fs::write(self.asset_path(name), body).unwrap();
    }

    pub fn remove_asset(&self, name: &str) {
        std::fs::remove_file(self.asset_path(name)).unwrap();
    }
}

/// A table of `count` Cells of one window, as Champion0, Champion1 and so on.
pub fn table_of(count: usize, window: &[&str]) -> Value {
    json!({
        "schema_version": 1,
        "patch_window": window,
        "cells": (0..count).map(|index| cell(&format!("Champion{index}"))).collect::<Vec<_>>(),
    })
}

pub fn cell(champion: &str) -> Value {
    json!({
        "champion": champion,
        "role": "TOP",
        "tier": "GOLD",
        "minutes": [{"minute": 0, "cs": 0.0, "total_gold": 500.0, "level": 1.0, "n": 100}],
    })
}
