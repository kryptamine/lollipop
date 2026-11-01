use std::ffi::{OsStr, OsString};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};
use tempfile::TempDir;

// RFC 8032, test 1: the public key is known without asking the code under test.
const BUCKET: &str = "test-bucket";
const SIGNING_SEED: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
pub const SIGNING_PUBLIC_KEY: &str =
    "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

/// The built binary against a fake `gh` and `rclone` first on its `PATH`, with the bucket in
/// a directory.
pub struct BucketBench {
    dir: TempDir,
    next_file: u32,
}

impl BucketBench {
    pub fn new() -> Self {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("bin")).unwrap();
        for (name, script) in [
            ("gh", include_str!("fake_gh.sh")),
            ("rclone", include_str!("fake_rclone.sh")),
        ] {
            let path = dir.path().join("bin").join(name);
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let bench = Self { dir, next_file: 0 };
        bench.set_reworks("[]");
        bench.set_signing_key(SIGNING_SEED);
        bench
    }

    pub fn set_reworks(&self, body: &str) {
        std::fs::write(self.reworks(), body).unwrap();
    }

    pub fn set_signing_key(&self, body: &str) {
        std::fs::write(self.signing_key(), body).unwrap();
    }

    fn signing_key(&self) -> PathBuf {
        self.dir.path().join("manifest-key")
    }

    pub fn reworks(&self) -> PathBuf {
        self.dir.path().join("reworks.json")
    }

    /// Runs of this `gh` command fail until `heal` is called.
    pub fn break_gh(&self, command: &str) {
        std::fs::write(self.dir.path().join(format!("fail-{command}")), "").unwrap();
    }

    pub fn break_rclone(&self, command: &str) {
        std::fs::write(self.dir.path().join(format!("fail-rclone-{command}")), "").unwrap();
    }

    pub fn fail_upload(&self, asset: &str) {
        std::fs::write(self.dir.path().join(format!("fail-upload-{asset}")), "").unwrap();
    }

    /// This object is listed, but its download writes nothing and still exits 0.
    pub fn lose_download(&self, asset: &str) {
        std::fs::write(self.dir.path().join(format!("lose-download-{asset}")), "").unwrap();
    }

    pub fn delete_asset(&self, asset: &str) {
        std::fs::remove_file(self.asset_path(asset)).unwrap();
    }

    /// The cache header the last upload of this asset carried.
    pub fn cache_header(&self, asset: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(format!("header-{asset}"))).unwrap()
    }

    pub fn heal(&self) {
        for entry in std::fs::read_dir(self.dir.path()).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with("fail-") || name.starts_with("lose-download-") {
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
            .args(["--bucket", BUCKET])
            .arg("--signing-key-file")
            .arg(self.signing_key())
            .env("PATH", path)
            .env("FAKE_GH_STATE", self.dir.path())
            .output()
            .unwrap()
    }

    pub fn manifest_public_key(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
            .arg("manifest-public-key")
            .arg("--signing-key-file")
            .arg(self.signing_key())
            .output()
            .unwrap()
    }

    // The schema's prefix is pinned here: it is the path the app reads.
    fn assets_dir(&self) -> PathBuf {
        self.dir.path().join("remote").join(BUCKET).join("v1")
    }

    pub fn asset_names(&self) -> Vec<String> {
        let Ok(assets) = std::fs::read_dir(self.assets_dir()) else {
            return Vec::new();
        };
        let mut names: Vec<String> = assets
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    pub fn asset_path(&self, name: &str) -> PathBuf {
        self.assets_dir().join(name)
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

    pub fn close_issues(&self) {
        let closed = self.dir.path().join("closed-issues");
        std::fs::rename(self.dir.path().join("issues"), closed).unwrap();
    }

    /// Corrupts a kept asset in place, as a bad upload or a hand edit would.
    pub fn overwrite_asset(&self, name: &str, body: &str) {
        std::fs::write(self.asset_path(name), body).unwrap();
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
