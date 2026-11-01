use std::path::PathBuf;
use std::time::Duration;

use lollipop_collector::{collect, Clock, CollectError, Config, SeedTier};
use rusqlite::Connection;
use tempfile::TempDir;

use super::fake_riot::FakeRiot;
use super::test_clock::TestClock;

pub const KEY: &str = "RGAPI-test-key";
const PATIENCE: Duration = Duration::from_secs(30);

/// A fake Riot, a test clock and a database, with runs of the collector against them.
/// Each run opens the database anew, and ending one drops it mid-flight, as a kill does.
pub struct Scenario {
    pub riot: FakeRiot,
    pub clock: TestClock,
    config: Config,
    pub dir: TempDir,
}

impl Scenario {
    pub async fn start(tier: SeedTier) -> Self {
        let clock = TestClock::new();
        let server_clock = clock.clone();
        let riot = FakeRiot::start(move || server_clock.now_ms()).await;
        let dir = TempDir::new().unwrap();
        let config = Config {
            tier,
            key: KEY.to_string(),
            database: dir.path().join("collector.sqlite3"),
            platform_url: riot.url.clone(),
            regional_url: riot.url.clone(),
        };
        Self {
            riot,
            clock,
            config,
            dir,
        }
    }

    /// Runs the collector until `done` holds, and fails the test if it stops first.
    pub async fn run_until(&self, done: impl Fn(&Self) -> bool) {
        if let Some(error) = self.run(done).await {
            panic!("the collector stopped: {error}");
        }
    }

    pub async fn run_until_all_visited(&self, matches: i64) {
        self.run_until(|scenario| scenario.visited() >= matches)
            .await;
    }

    pub async fn run_until_stopped(&self) -> CollectError {
        self.run(|_| false).await.unwrap()
    }

    async fn run(&self, done: impl Fn(&Self) -> bool) -> Option<CollectError> {
        let until_done = async {
            while !done(self) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        };
        let raced = async {
            tokio::select! {
                biased;
                error = collect(&self.config, &self.clock) => Some(error),
                () = until_done => None,
            }
        };
        tokio::time::timeout(PATIENCE, raced).await.unwrap()
    }

    pub fn point_at(&mut self, url: &str) {
        self.config.platform_url = url.to_string();
        self.config.regional_url = url.to_string();
    }

    pub fn database(&self) -> PathBuf {
        self.config.database.clone()
    }

    pub fn count(&self, sql: &str) -> i64 {
        if !self.database().exists() {
            return 0;
        }
        let connection = Connection::open(self.database()).unwrap();
        connection.query_row(sql, [], |row| row.get(0)).unwrap_or(0)
    }

    pub fn stored(&self) -> i64 {
        self.count("SELECT count(*) FROM matches")
    }

    pub fn dropped(&self) -> i64 {
        self.count("SELECT count(*) FROM dropped_matches")
    }

    pub fn visited(&self) -> i64 {
        self.stored() + self.dropped()
    }

    pub fn stored_ids(&self) -> Vec<String> {
        let connection = Connection::open(self.database()).unwrap();
        let mut statement = connection
            .prepare("SELECT match_id FROM matches ORDER BY match_id")
            .unwrap();
        let ids = statement.query_map([], |row| row.get(0)).unwrap();
        ids.map(Result::unwrap).collect()
    }
}
