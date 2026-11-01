//! Drives the poll loop against a local HTTP server standing in for League.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::sync::Arc;

use lollipop_core::live::Snapshot;
use tokio::net::TcpListener;

use support::game_clock::GameClock;
use support::polling::{first_update, start_polling};
use support::recorded_game::RecordedGame;
use support::server::serve;

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
    // The recording opens on the loading screen, before GameStart.
    assert_eq!(updates.recv().await.unwrap().1, None);

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
          "events": { "Events": [{ "EventName": "GameStart" }] },
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
          "events": { "Events": [{ "EventName": "GameStart" }] },
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
          "events": { "Events": [{ "EventName": "GameStart" }] },
          "gameData": { "gameTime": 120.0 }
        }"#,
    )
    .await;
    assert_eq!(update, None);
}

// A 200 can come about 56 s before GameStart, with the player already in the payload.
#[tokio::test]
async fn a_game_before_game_start_has_no_snapshot() {
    let update = update_for(
        r#"{
          "activePlayer": { "riotId": "Me#EUW" },
          "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 0 } }],
          "events": { "Events": [] },
          "gameData": { "gameTime": 0.02 }
        }"#,
    )
    .await;
    assert_eq!(update, None);
}

// The API stays up for a moment after GameEnd, with the final scores still served.
#[tokio::test]
async fn a_game_after_game_end_has_no_snapshot() {
    let update = update_for(
        r#"{
          "activePlayer": { "riotId": "Me#EUW" },
          "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 220 } }],
          "events": { "Events": [{ "EventName": "GameStart" }, { "EventName": "GameEnd" }] },
          "gameData": { "gameTime": 1995.2 }
        }"#,
    )
    .await;
    assert_eq!(update, None);
}

// One load in three served a 200 about 0.25 s before the player could be matched.
#[tokio::test]
async fn a_game_with_no_active_player_has_no_snapshot() {
    let update = update_for(
        r#"{
          "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 0 } }],
          "events": { "Events": [{ "EventName": "GameStart" }] },
          "gameData": { "gameTime": 0.02 }
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
