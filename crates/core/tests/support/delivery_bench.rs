use std::sync::Arc;
use std::time::Duration;

use lollipop_core::delivery::{keep_table_current, ManifestKey, TableChannel, TableFile};
use lollipop_core::live::LiveGame;
use tempfile::TempDir;
use tokio::sync::mpsc;

use super::benchmark_file::{cell, flat_minutes, table_of};
use super::fake_bucket::{manifest_key, FakeBucket};
use super::fake_lcu::FakeLcu;
use super::game_clock::GameClock;
use super::marker_game::{CHAMPION, ROLE, TIER};
use super::polling::start_playing_on;
use super::staged_game::{Stage, StagedGame};

/// A different table per `cs`: all that delivery looks at is which file is live.
pub fn table_with_cs(cs: f64) -> String {
    table_of(&[cell(CHAMPION, ROLE, TIER, &flat_minutes(cs, 1000.0, 10.0))])
}

/// The real poll loop and the real download loop, sharing one table file in a directory
/// that does not exist yet, against a fake League and a fake bucket.
pub struct DeliveryBench {
    pub bucket: FakeBucket,
    pub game: StagedGame,
    pub lcu: FakeLcu,
    pub table: Arc<TableFile>,
    updates: Option<mpsc::UnboundedReceiver<(f64, Option<LiveGame>)>>,
    _dir: TempDir,
}

impl DeliveryBench {
    pub async fn at(stage: Stage) -> Self {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("data").join("benchmark-table.json");
        Self {
            bucket: FakeBucket::start().await,
            game: StagedGame::at(stage).await,
            lcu: FakeLcu::start().await,
            table: Arc::new(TableFile::new(path)),
            updates: None,
            _dir: dir,
        }
    }

    pub fn write_live(&self, body: &str) {
        std::fs::create_dir_all(self.table.path().parent().unwrap()).unwrap();
        std::fs::write(self.table.path(), body).unwrap();
    }

    pub fn live(&self) -> Option<String> {
        std::fs::read_to_string(self.table.path()).ok()
    }

    pub fn start_polling(&mut self) {
        self.updates = Some(start_playing_on(
            self.game.url.clone(),
            &self.lcu.lockfile(),
            Arc::clone(&self.table),
            GameClock::starting_at(0.0),
        ));
    }

    pub fn start_downloads(&self) {
        self.start_downloads_on(self.bucket.channel());
    }

    pub fn start_downloads_on(&self, channel: TableChannel) {
        self.start_downloads_with(channel, manifest_key());
    }

    pub fn start_downloads_with(&self, channel: TableChannel, key: ManifestKey) {
        let table = Arc::clone(&self.table);
        tokio::spawn(async move { keep_table_current(&table, &channel, &key).await });
    }

    /// The next update that shows a running game, so the table has been read for it.
    pub async fn next_game_update(&mut self) -> LiveGame {
        let updates = self.updates.as_mut().unwrap();
        loop {
            if let (_, Some(game)) = updates.recv().await.unwrap() {
                return game;
            }
        }
    }

    /// Long enough for a finished download to be staged, and for a swap, if one is coming.
    pub async fn settle(&mut self) {
        let updates = self.updates.as_mut();
        tokio::time::sleep(Duration::from_millis(50)).await;
        if let Some(updates) = updates {
            while updates.try_recv().is_ok() {}
        }
    }
}

/// Waits for what the background work will do, and fails rather than hang.
pub async fn until(what: &str, done: impl Fn() -> bool) {
    for _ in 0..5000 {
        if done() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    panic!("timed out waiting until {what}");
}
