//! The menu's status line, with League's Live Client Data and client faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;

use lollipop_core::live::status_line;
use serde_json::json;
use tempfile::TempDir;

use support::fake_lcu::{
    champ_select, ranked, stale_lockfile, FakeLcu, CHAMP_SELECT, RANKED, SESSION, VERSION,
};
use support::polling::{serve_in_order, start_watching};
use support::scripted_game::{as_master_yi, loading_body};

async fn status_for(body: String, lockfile: &Path) -> String {
    let url = serve_in_order(vec![Some(body)]).await;
    let update = start_watching(url, lockfile).recv().await.unwrap();
    status_line(update.as_ref().map(|game| game.inputs.as_ref()))
}

async fn status_as_mid(lcu: &FakeLcu) -> String {
    status_for(as_master_yi(30.0, "MIDDLE"), &lcu.lockfile()).await
}

#[tokio::test]
async fn names_what_was_detected() {
    let lcu = FakeLcu::start().await;
    assert_eq!(status_as_mid(&lcu).await, "Master Yi · Mid · Gold");
}

#[tokio::test]
async fn a_loading_game_is_not_yet_a_game() {
    let lcu = FakeLcu::start().await;
    let status = status_for(loading_body(0.02), &lcu.lockfile()).await;
    assert_eq!(status, "No game running");
}

#[tokio::test]
async fn names_every_role_and_tier_the_way_the_menu_does() {
    let roles = [
        ("TOP", "Top"),
        ("JUNGLE", "Jungle"),
        ("BOTTOM", "Bot"),
        ("UTILITY", "Support"),
    ];
    for (position, label) in roles {
        let lcu = FakeLcu::start().await;
        let status = status_for(as_master_yi(30.0, position), &lcu.lockfile()).await;
        assert_eq!(status, format!("Master Yi · {label} · Gold"));
    }
    let tiers = [
        ("IRON", "Iron"),
        ("BRONZE", "Bronze"),
        ("SILVER", "Silver"),
        ("PLATINUM", "Platinum"),
        ("EMERALD", "Emerald"),
        ("DIAMOND", "Diamond"),
        ("MASTER", "Master"),
        ("GRANDMASTER", "Grandmaster"),
        ("CHALLENGER", "Challenger"),
    ];
    for (tier, label) in tiers {
        let lcu = FakeLcu::start().await;
        lcu.set(RANKED, &ranked(tier));
        assert_eq!(
            status_as_mid(&lcu).await,
            format!("Master Yi · Mid · {label}")
        );
    }
}

// Each of these gets its own rule from later Widgets, so none may read like another.
#[tokio::test]
async fn an_unavailable_client_a_missing_tier_and_an_unranked_player_differ() {
    let no_client = TempDir::new().unwrap();
    let unavailable = status_for(
        as_master_yi(30.0, "MIDDLE"),
        &no_client.path().join("lockfile"),
    )
    .await;

    let missing = FakeLcu::start().await;
    missing.remove(RANKED);

    let unranked = FakeLcu::start().await;
    unranked.set(RANKED, &ranked("NONE"));

    assert_eq!(unavailable, "Master Yi · Mid · League client unavailable");
    assert_eq!(
        status_as_mid(&missing).await,
        "Master Yi · Mid · Tier unknown"
    );
    assert_eq!(status_as_mid(&unranked).await, "Master Yi · Mid · Unranked");
}

#[tokio::test]
async fn gives_the_reason_for_a_missing_role() {
    let lcu = FakeLcu::start().await;
    let status = status_for(as_master_yi(30.0, "NONE"), &lcu.lockfile()).await;
    assert_eq!(status, "Master Yi · Role unknown · Gold");
}

#[tokio::test]
async fn gives_the_reason_for_a_missing_queue_and_patch() {
    let lcu = FakeLcu::start().await;
    lcu.remove(SESSION);
    lcu.remove(VERSION);
    assert_eq!(
        status_as_mid(&lcu).await,
        "Master Yi · Mid · Gold · Queue unknown · Patch unknown"
    );
}

#[tokio::test]
async fn gives_the_reason_for_a_missing_champion() {
    let lcu = FakeLcu::start().await;
    let body = json!({
        "activePlayer": { "riotId": "Me#EUW", "currentGold": 500.0 },
        "allPlayers": [{
            "riotId": "Me#EUW", "position": "MIDDLE", "level": 1, "items": [],
            "scores": { "creepScore": 0 },
        }],
        "events": { "Events": [{ "EventName": "GameStart" }] },
        "gameData": { "gameTime": 30.0 },
    });
    let status = status_for(body.to_string(), &lcu.lockfile()).await;
    assert_eq!(status, "Champion unknown · Mid · Gold");
}

#[tokio::test]
async fn a_stale_lockfile_reads_as_unavailable() {
    let dir = TempDir::new().unwrap();
    let lockfile = stale_lockfile(dir.path()).await;
    let status = status_for(as_master_yi(30.0, "NONE"), &lockfile).await;
    assert_eq!(
        status,
        "Master Yi · Role unknown · League client unavailable"
    );
}

#[tokio::test]
async fn takes_the_role_from_champ_select_with_no_position() {
    let lcu = FakeLcu::start().await;
    lcu.set(CHAMP_SELECT, &champ_select("middle"));
    let status = status_for(as_master_yi(30.0, ""), &lcu.lockfile()).await;
    assert_eq!(status, "Master Yi · Mid · Gold");
}
