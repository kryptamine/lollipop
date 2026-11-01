//! Queue, patch, role and tier, read through a fake League client's lockfile.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;
use std::sync::Arc;

use lollipop_core::game_inputs::{
    ClientInputs, ClientReading, GameInputs, Patch, Role, Tier, TierReading,
};
use serde_json::json;
use tempfile::TempDir;

use support::fake_lcu::{
    champ_select, ranked, stale_lockfile, FakeLcu, CHAMP_SELECT, RANKED, SESSION, VERSION,
};
use support::polling::{serve_in_order, start_watching};
use support::recorded_game::RecordedGame;
use support::scripted_game::{as_master_yi, loading_body};
use support::server::serve;

async fn inputs_for(position: &str, lockfile: &Path) -> Arc<GameInputs> {
    let url = serve_in_order(vec![Some(as_master_yi(30.0, position))]).await;
    let update = start_watching(url, lockfile).recv().await.unwrap();
    update.unwrap().inputs
}

async fn reading_of(lcu: &FakeLcu) -> ClientReading {
    let inputs = inputs_for("MIDDLE", &lcu.lockfile()).await;
    match inputs.client {
        ClientInputs::Read(reading) => reading,
        ClientInputs::Unavailable => panic!("the client should have been read"),
    }
}

async fn tier_of(lcu: &FakeLcu) -> TierReading {
    reading_of(lcu).await.tier
}

#[tokio::test]
async fn reads_queue_patch_and_tier_from_the_client() {
    let lcu = FakeLcu::start().await;
    assert_eq!(
        reading_of(&lcu).await,
        ClientReading {
            queue: Some(420),
            patch: Some(Patch {
                major: 16,
                minor: 21
            }),
            tier: TierReading::Ranked(Tier::Gold),
        }
    );
}

// The recorded game is Warwick in the jungle, on the patch string the probe captured.
#[tokio::test]
async fn reads_a_recorded_game_with_the_recorded_version_string() {
    let lcu = FakeLcu::start().await;
    lcu.set(
        VERSION,
        r#""16.21.8342754+branch.releases-16-21.code.public.content.release.anticheat.vanguard""#,
    );
    let game = RecordedGame::load();
    let url = serve(Box::new(move || game.payload_at(1.0))).await;

    let inputs = start_watching(url, &lcu.lockfile())
        .recv()
        .await
        .unwrap()
        .unwrap()
        .inputs;

    assert_eq!(inputs.champion.as_deref(), Some("Warwick"));
    assert_eq!(inputs.role, Some(Role::Jungle));
    let ClientInputs::Read(reading) = &inputs.client else {
        panic!("the client should have been read");
    };
    let expected = Patch {
        major: 16,
        minor: 21,
    };
    assert_eq!(reading.patch, Some(expected));
}

// A lane swap: what the game says now outranks what champ select assigned.
#[tokio::test]
async fn position_wins_over_assigned_position() {
    let lcu = FakeLcu::start().await;
    lcu.set(CHAMP_SELECT, &champ_select("top"));
    let inputs = inputs_for("JUNGLE", &lcu.lockfile()).await;
    assert_eq!(inputs.role, Some(Role::Jungle));
    assert_eq!(lcu.requests_for(CHAMP_SELECT), 0);
}

#[tokio::test]
async fn assigned_position_fills_an_empty_or_none_position() {
    for position in ["", "NONE"] {
        let lcu = FakeLcu::start().await;
        lcu.set(CHAMP_SELECT, &champ_select("utility"));
        let inputs = inputs_for(position, &lcu.lockfile()).await;
        assert_eq!(inputs.role, Some(Role::Support), "position {position:?}");
    }
}

// Practice Tool: `position` is NONE and champ select assigns nothing. Never guessed.
#[tokio::test]
async fn no_role_anywhere_is_an_unknown_role() {
    let lcu = FakeLcu::start().await;
    lcu.set(CHAMP_SELECT, &champ_select(""));
    assert_eq!(inputs_for("NONE", &lcu.lockfile()).await.role, None);

    // The session of champ select is gone once it is over.
    let without_session = FakeLcu::start().await;
    let inputs = inputs_for("NONE", &without_session.lockfile()).await;
    assert_eq!(inputs.role, None);
    assert!(matches!(inputs.client, ClientInputs::Read(_)));
}

#[tokio::test]
async fn an_absent_or_unreadable_lockfile_is_an_unavailable_client() {
    let dir = TempDir::new().unwrap();
    let absent = dir.path().join("absent");
    let garbage = dir.path().join("garbage");
    std::fs::write(&garbage, "not a lockfile").unwrap();

    for lockfile in [absent, garbage] {
        let inputs = inputs_for("MIDDLE", &lockfile).await;
        assert_eq!(inputs.client, ClientInputs::Unavailable);
        // The role does not need the client, so losing it must not lose the role.
        assert_eq!(inputs.role, Some(Role::Mid));
    }
}

// The lockfile outlives a crashed client, so its presence proves nothing.
#[tokio::test]
async fn a_lockfile_with_no_client_behind_it_is_an_unavailable_client() {
    let dir = TempDir::new().unwrap();
    let lockfile = stale_lockfile(dir.path()).await;
    let inputs = inputs_for("MIDDLE", &lockfile).await;
    assert_eq!(inputs.client, ClientInputs::Unavailable);
}

// The password in the lockfile is what lets every request through.
#[tokio::test]
async fn a_wrong_password_is_an_unavailable_client() {
    let lcu = FakeLcu::with_password("not-the-password").await;
    let inputs = inputs_for("MIDDLE", &lcu.lockfile()).await;
    assert_eq!(inputs.client, ClientInputs::Unavailable);
    assert_eq!(lcu.requests_for(SESSION), 1);
}

// If only the tier cannot be read, Timing hints keep working on queue and patch.
#[tokio::test]
async fn a_failed_tier_request_keeps_queue_and_patch() {
    let lcu = FakeLcu::start().await;
    lcu.remove(RANKED);
    let reading = reading_of(&lcu).await;
    assert_eq!(reading.tier, TierReading::Unknown);
    assert_eq!(reading.queue, Some(420));
    assert!(reading.patch.is_some());
}

#[tokio::test]
async fn no_solo_duo_entry_is_an_unknown_tier() {
    let lcu = FakeLcu::start().await;
    let flex_only = json!({ "queues": [{ "queueType": "RANKED_FLEX_SR", "tier": "GOLD" }] });
    lcu.set(RANKED, &flex_only.to_string());
    assert_eq!(tier_of(&lcu).await, TierReading::Unknown);
}

#[tokio::test]
async fn a_tier_nobody_knows_is_unknown_not_guessed() {
    let lcu = FakeLcu::start().await;
    lcu.set(RANKED, &ranked("WOOD"));
    assert_eq!(tier_of(&lcu).await, TierReading::Unknown);
}

#[tokio::test]
async fn a_null_tier_is_unknown() {
    let lcu = FakeLcu::start().await;
    let null_tier = json!({ "queues": [{ "queueType": "RANKED_SOLO_5x5", "tier": null }] });
    lcu.set(RANKED, &null_tier.to_string());
    assert_eq!(tier_of(&lcu).await, TierReading::Unknown);
}

#[tokio::test]
async fn every_way_of_writing_no_rank_is_unranked() {
    for tier in ["", "NONE", "UNRANKED"] {
        let lcu = FakeLcu::start().await;
        lcu.set(RANKED, &ranked(tier));
        assert_eq!(tier_of(&lcu).await, TierReading::Unranked, "tier {tier:?}");
    }
}

// Flex is listed first and at a higher tier: Cells are built from solo/duo only.
#[tokio::test]
async fn the_tier_is_the_solo_duo_entrys() {
    let lcu = FakeLcu::start().await;
    lcu.set(RANKED, &ranked("SILVER"));
    assert_eq!(tier_of(&lcu).await, TierReading::Ranked(Tier::Silver));
}

// An unknown patch hides Markers; a version misread as a patch would pick a wrong table.
#[tokio::test]
async fn a_version_that_does_not_read_is_an_unknown_patch() {
    for version in [r#""""#, r#""unknown""#, r#""16""#, r#""16.x.1""#] {
        let lcu = FakeLcu::start().await;
        lcu.set(VERSION, version);
        assert_eq!(reading_of(&lcu).await.patch, None, "version {version}");
    }
}

// The client answers -1 when it has no queue, which is no queue rather than a parse error.
#[tokio::test]
async fn a_session_with_no_queue_is_an_unknown_queue() {
    let lcu = FakeLcu::start().await;
    lcu.set(SESSION, r#"{ "gameData": { "queue": { "id": -1 } } }"#);
    assert_eq!(reading_of(&lcu).await.queue, None);
}

// Later polls must see the game's start: a tier or role that moves would make Markers jump.
#[tokio::test]
async fn nothing_read_at_game_start_changes_during_the_game() {
    let lcu = FakeLcu::start().await;
    let url = serve_in_order(vec![
        Some(as_master_yi(30.0, "MIDDLE")),
        Some(as_master_yi(31.0, "JUNGLE")),
        // A failed poll in mid-game must not count as a new game.
        None,
        Some(as_master_yi(33.0, "TOP")),
    ])
    .await;
    let mut updates = start_watching(url, &lcu.lockfile());

    let first = updates.recv().await.unwrap().unwrap().inputs;
    lcu.set(RANKED, &ranked("DIAMOND"));
    lcu.set(SESSION, r#"{ "gameData": { "queue": { "id": 440 } } }"#);

    let second = updates.recv().await.unwrap().unwrap().inputs;
    assert!(updates.recv().await.unwrap().is_none());
    let fourth = updates.recv().await.unwrap().unwrap().inputs;

    assert_eq!(first.role, Some(Role::Mid));
    assert_eq!(*second, *first);
    assert_eq!(*fourth, *first);
    for path in [SESSION, VERSION, RANKED] {
        assert_eq!(lcu.requests_for(path), 1, "requests for {path}");
    }
}

#[tokio::test]
async fn the_next_game_reads_again_after_a_loading_screen() {
    let lcu = FakeLcu::start().await;
    let url = serve_in_order(vec![
        Some(as_master_yi(30.0, "MIDDLE")),
        Some(loading_body(0.02)),
        Some(as_master_yi(40.0, "JUNGLE")),
    ])
    .await;
    let mut updates = start_watching(url, &lcu.lockfile());

    let first = updates.recv().await.unwrap().unwrap().inputs;
    lcu.set(RANKED, &ranked("DIAMOND"));
    assert!(updates.recv().await.unwrap().is_none());
    let next = updates.recv().await.unwrap().unwrap().inputs;

    assert_eq!(first.role, Some(Role::Mid));
    assert_eq!(next.role, Some(Role::Jungle));
    let ClientInputs::Read(reading) = &next.client else {
        panic!("the client should have been read");
    };
    assert_eq!(reading.tier, TierReading::Ranked(Tier::Diamond));
}

// Only a new game turns the clock back, even if its loading screen was never seen.
#[tokio::test]
async fn a_clock_turned_back_is_a_new_game() {
    let lcu = FakeLcu::start().await;
    let url = serve_in_order(vec![
        Some(as_master_yi(900.0, "MIDDLE")),
        Some(as_master_yi(2.0, "JUNGLE")),
    ])
    .await;
    let mut updates = start_watching(url, &lcu.lockfile());
    let first = updates.recv().await.unwrap().unwrap().inputs;
    let next = updates.recv().await.unwrap().unwrap().inputs;
    assert_eq!(first.role, Some(Role::Mid));
    assert_eq!(next.role, Some(Role::Jungle));
}

// A 200 can come about 56 s before GameStart, when a read could catch the client mid-load.
#[tokio::test]
async fn the_client_is_not_read_before_game_start() {
    let lcu = FakeLcu::start().await;
    let url = serve_in_order(vec![Some(loading_body(0.02))]).await;
    let update = start_watching(url, &lcu.lockfile()).recv().await.unwrap();
    assert!(update.is_none());
    assert_eq!(lcu.requests_for(SESSION), 0);
}
