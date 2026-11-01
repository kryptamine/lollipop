use lollipop_core::live::{poll, Snapshot};
use tokio::sync::mpsc;

use super::game_clock::GameClock;

/// Runs the poll loop on `clock` and returns updates, stamped with game time, as they arrive.
pub fn start_polling(
    url: String,
    clock: GameClock,
) -> mpsc::UnboundedReceiver<(f64, Option<Snapshot>)> {
    let (updates, received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let stamp = clock.clone();
        poll(&url, &clock, move |update| {
            let _ = updates.send((stamp.now(), update));
        })
        .await;
    });
    received
}

pub async fn first_update(url: String) -> Option<Snapshot> {
    let mut updates = start_polling(url, GameClock::starting_at(0.0));
    updates.recv().await.unwrap().1
}
