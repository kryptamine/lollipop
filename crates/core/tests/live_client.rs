//! Drives the poll loop against a local HTTP server standing in for League.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

mod support;

use std::sync::Arc;

use lollipop_core::live::{poll, Snapshot};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use support::game_clock::GameClock;
use support::recorded_game::RecordedGame;
use support::server::serve;

/// Runs the poll loop on `clock` and returns updates, stamped with game time, as they arrive.
fn start_polling(
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

async fn first_update(url: String) -> Option<Snapshot> {
    let mut updates = start_polling(url, GameClock::starting_at(0.0));
    updates.recv().await.unwrap().1
}

async fn update_for(body: &'static str) -> Option<Snapshot> {
    first_update(serve(Box::new(|| Some(body.to_string()))).await).await
}

// Protects the whole path from HTTP to Snapshot on real data. The values were read
// from recorder.ndjson by hand, so a parser that drifts from the recording fails.
#[tokio::test]
async fn replays_a_recorded_game_on_game_time() {
    let game = Arc::new(RecordedGame::load());
    let clock = GameClock::starting_at(game.first_game_time());
    let server_clock = clock.clone();
    let server_game = Arc::clone(&game);
    let url = serve(Box::new(move || server_game.payload_at(server_clock.now()))).await;

    let mut updates = start_polling(url, clock);
    let mut cs_at_minute = Vec::new();
    let mut last_cs = None;
    let mut next_minute = 5.0;
    let ended_at = loop {
        let (game_time, update) = updates.recv().await.unwrap();
        let Some(snapshot) = update else {
            break game_time;
        };
        if game_time >= next_minute * 60.0 {
            cs_at_minute.push(snapshot.cs);
            next_minute += 5.0;
        }
        last_cs = Some(snapshot.cs);
    };

    assert_eq!(cs_at_minute, [30, 70, 100, 140, 180, 200]);
    assert_eq!(last_cs, Some(220));
    assert!((1995.0..1997.0).contains(&ended_at), "ended at {ended_at}");
}

// Allied and enemy scores share the payload and must never be shown as the player's.
#[tokio::test]
async fn reads_only_the_active_players_cs() {
    let update = update_for(
        r#"{
          "activePlayer": { "riotId": "Me#EUW" },
          "allPlayers": [
            { "riotId": "Enemy#EUW", "scores": { "creepScore": 200 } },
            { "riotId": "Me#EUW", "scores": { "creepScore": 60 } }
          ],
          "gameData": { "gameTime": 600.0 }
        }"#,
    )
    .await;
    assert_eq!(update, Some(Snapshot { cs: 60 }));
}

#[tokio::test]
async fn falls_back_to_the_summoner_name() {
    let update = update_for(
        r#"{
          "activePlayer": { "summonerName": "Me" },
          "allPlayers": [{ "summonerName": "Me", "scores": { "creepScore": 5 } }],
          "gameData": { "gameTime": 120.0 }
        }"#,
    )
    .await;
    assert_eq!(update, Some(Snapshot { cs: 5 }));
}

// A spectator payload has no ids, and two missing ids are not the same player.
#[tokio::test]
async fn missing_ids_never_match_each_other() {
    let update = update_for(
        r#"{
          "activePlayer": {},
          "allPlayers": [{ "scores": { "creepScore": 5 } }],
          "gameData": { "gameTime": 120.0 }
        }"#,
    )
    .await;
    assert_eq!(update, None);
}

#[tokio::test]
async fn a_body_that_is_not_a_game_has_no_snapshot() {
    assert_eq!(
        update_for(r#"{"errorCode":"RESOURCE_NOT_FOUND"}"#).await,
        None
    );
    assert_eq!(update_for("not json").await, None);
}

// League refuses connections whenever no game is running.
#[tokio::test]
async fn a_refused_connection_has_no_snapshot() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    assert_eq!(first_update(url).await, None);
}
