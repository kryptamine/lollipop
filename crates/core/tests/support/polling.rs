use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use lollipop_core::delivery::TableFile;
use lollipop_core::live::{poll, LiveGame, LiveUpdate, Snapshot};
use tokio::sync::mpsc;

use super::game_clock::GameClock;
use super::server::serve;

const NO_TABLE: &str = "/nonexistent/benchmark-table.json";

/// Runs the poll loop on `clock` and returns updates, stamped with game time, as they arrive.
pub fn start_polling(
    url: String,
    clock: GameClock,
) -> mpsc::UnboundedReceiver<(f64, Option<Snapshot>)> {
    let (updates, received) = mpsc::unbounded_channel();
    let no_client = PathBuf::from("/nonexistent/lockfile");
    tokio::spawn(async move {
        let stamp = clock.clone();
        poll(
            &url,
            &no_client,
            &TableFile::new(NO_TABLE.into()),
            &clock,
            move |update: LiveUpdate| {
                let snapshot = update.game.map(|game| game.snapshot);
                let _ = updates.send((stamp.now(), snapshot));
            },
        )
        .await;
    });
    received
}

pub async fn first_update(url: String) -> Option<Snapshot> {
    let mut updates = start_polling(url, GameClock::starting_at(0.0));
    updates.recv().await.unwrap().1
}

/// Serves `bodies` in order, then drops every connection; `None` drops one poll's.
pub async fn serve_in_order(bodies: Vec<Option<String>>) -> String {
    let bodies = Mutex::new(bodies.into_iter());
    serve(Box::new(move || bodies.lock().unwrap().next().flatten())).await
}

/// Polls with the League client's lockfile at `lockfile`.
pub fn start_watching(url: String, lockfile: &Path) -> mpsc::UnboundedReceiver<Option<LiveGame>> {
    let (updates, received) = mpsc::unbounded_channel();
    let lockfile = lockfile.to_path_buf();
    tokio::spawn(async move {
        let clock = GameClock::starting_at(0.0);
        poll(
            &url,
            &lockfile,
            &TableFile::new(NO_TABLE.into()),
            &clock,
            move |update: LiveUpdate| {
                let _ = updates.send(update.game);
            },
        )
        .await;
    });
    received
}

/// Polls with a League client and a Benchmark table on disk, stamping updates with game time.
pub fn start_playing(
    url: String,
    lockfile: &Path,
    benchmark_table: &Path,
    clock: GameClock,
) -> mpsc::UnboundedReceiver<(f64, Option<LiveGame>)> {
    let table = Arc::new(TableFile::new(benchmark_table.to_path_buf()));
    start_playing_on(url, lockfile, table, clock)
}

/// As `start_playing`, on a table file the test shares with its downloads.
pub fn start_playing_on(
    url: String,
    lockfile: &Path,
    table: Arc<TableFile>,
    clock: GameClock,
) -> mpsc::UnboundedReceiver<(f64, Option<LiveGame>)> {
    let (updates, received) = mpsc::unbounded_channel();
    let lockfile = lockfile.to_path_buf();
    tokio::spawn(async move {
        let stamp = clock.clone();
        poll(
            &url,
            &lockfile,
            &table,
            &clock,
            move |update: LiveUpdate| {
                let _ = updates.send((stamp.now(), update.game));
            },
        )
        .await;
    });
    received
}

/// Serves `bodies` in order and returns, poll by poll, whether each saw League
/// between games.
pub async fn between_games_over(bodies: Vec<Option<&str>>) -> Vec<bool> {
    let polls = bodies.len();
    let url = serve_in_order(bodies.into_iter().map(|b| b.map(String::from)).collect()).await;
    let (updates, mut received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        poll(
            &url,
            Path::new("/nonexistent/lockfile"),
            &TableFile::new(NO_TABLE.into()),
            &GameClock::starting_at(0.0),
            move |update: LiveUpdate| {
                let _ = updates.send(update.between_games);
            },
        )
        .await;
    });
    let mut seen = Vec::new();
    for _ in 0..polls {
        seen.push(received.recv().await.unwrap());
    }
    seen
}
