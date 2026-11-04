//! The built binary, killed for real and started again.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tempfile::TempDir;

use support::fake_riot::FakeRiot;
use support::game::Game;

const PATIENCE: Duration = Duration::from_secs(60);

fn now_ms() -> u64 {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    u64::try_from(since.as_millis()).unwrap()
}

fn spawn(url: &str, dir: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
        .args(["collect", "--tier", "gold"])
        .arg("--database")
        .arg(dir.join("collector.sqlite3"))
        .arg("--key-file")
        .arg(dir.join("riot-key"))
        .args(["--platform-url", url, "--regional-url", url])
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

fn stored(dir: &Path) -> i64 {
    let Ok(connection) = Connection::open(dir.join("collector.sqlite3")) else {
        return 0;
    };
    connection
        .query_row("SELECT count(*) FROM matches", [], |row| row.get(0))
        .unwrap_or(0)
}

async fn wait_until(done: impl Fn() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < PATIENCE, "timed out");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

// Protects the one thing a dropped future cannot show: a real SIGKILL against the real
// database files, with the binary's own argument handling on the way in.
#[tokio::test]
async fn resumes_after_a_kill() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("riot-key"), "RGAPI-from-file\n").unwrap();
    let riot = FakeRiot::start(now_ms).await;
    // Slow enough for the kill to land before the run can finish.
    riot.set_rate_limit("30:1");
    let games: Vec<Game> = (0..30).map(|n| Game::new(&format!("EUW1_{n}"))).collect();
    for (index, games) in games.chunks(3).enumerate() {
        riot.add_seed("GOLD", &format!("seed-{index}"), games);
    }

    let mut first = spawn(&riot.url, dir.path());
    wait_until(|| stored(dir.path()) >= 5).await;
    first.kill().unwrap();
    first.wait().unwrap();
    let before_restart = stored(dir.path());
    assert!(before_restart < 30);

    let mut second = spawn(&riot.url, dir.path());
    wait_until(|| stored(dir.path()) == 30).await;
    second.kill().unwrap();
    second.wait().unwrap();

    for request in riot.requests() {
        assert_eq!(request.token.as_deref(), Some("RGAPI-from-file"));
    }
    // One match may be in flight at the kill, and every other is fetched once.
    let fetched = riot.request_count("/timeline");
    assert!((30..=31).contains(&fetched), "{fetched}");
}

#[tokio::test]
async fn a_missing_key_file_fails_with_a_message() {
    let dir = TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
        .args(["collect", "--tier", "gold", "--key-file"])
        .arg(dir.path().join("riot-key"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("riot-key"), "{stderr}");
    // A message printed as Debug comes quoted, with its newlines escaped.
    assert!(!stderr.starts_with("Error: \""), "{stderr}");
}

#[tokio::test]
async fn an_unknown_tier_is_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_lollipop-collector"))
        .args(["collect", "--tier", "wood"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}
