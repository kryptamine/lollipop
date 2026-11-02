//! Retention: raw matches outside the Patch window go when it shifts.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_collector::SeedTier;

use support::game::Game;
use support::scenario::Scenario;

async fn store(scenario: &Scenario, seed: &str, game: Game) {
    let visited = scenario.visited();
    scenario.riot.add_seed("GOLD", seed, &[game]);
    scenario.run_until_all_visited(visited + 1).await;
}

async fn scenario_on_patches(games: &[(&str, &str)]) -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    for (index, (id, version)) in games.iter().enumerate() {
        store(
            &scenario,
            &format!("seed-{index}"),
            Game::on_patch(id, version),
        )
        .await;
    }
    scenario
}

fn ids(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

// The window is the last two patches. Everything of the old patch goes, child rows too:
// the spec's size estimate rests on this.
#[tokio::test]
async fn a_new_patch_deletes_the_matches_it_pushes_out() {
    let scenario = scenario_on_patches(&[
        ("EUW1_20A", "16.20.824.8524"),
        ("EUW1_20B", "16.20.900.1"),
        ("EUW1_21", "16.21.824.8524"),
    ])
    .await;
    assert_eq!(scenario.stored(), 3);

    store(
        &scenario,
        "seed-new",
        Game::on_patch("EUW1_22", "16.22.1.1"),
    )
    .await;

    assert_eq!(scenario.stored_ids(), ids(&["EUW1_21", "EUW1_22"]));
    assert_eq!(scenario.count("SELECT count(*) FROM participants"), 20);
    assert_eq!(scenario.count("SELECT count(*) FROM minute_values"), 60);
    assert_eq!(scenario.count("SELECT count(*) FROM first_kills"), 10);
}

// A hotfix keeps the patch number, so it must not read as a new patch.
#[tokio::test]
async fn a_hotfix_on_the_same_patch_does_not_shift_the_window() {
    let scenario = scenario_on_patches(&[
        ("EUW1_20", "16.20.824.8524"),
        ("EUW1_21A", "16.21.824.8524"),
        ("EUW1_21B", "16.21.999.1"),
    ])
    .await;
    assert_eq!(
        scenario.stored_ids(),
        ids(&["EUW1_20", "EUW1_21A", "EUW1_21B"])
    );
}

// Numeric, as the text "16.9" sorts above "16.10", and the Season turns over at 17.1.
#[tokio::test]
async fn patches_compare_as_numbers_across_a_season() {
    let scenario = scenario_on_patches(&[
        ("EUW1_9", "16.9.1.1"),
        ("EUW1_10", "16.10.1.1"),
        ("EUW1_24", "16.24.1.1"),
        ("EUW1_17_1", "17.1.1.1"),
    ])
    .await;
    assert_eq!(scenario.stored_ids(), ids(&["EUW1_17_1", "EUW1_24"]));
}

// An old game is worth neither its timeline call nor a place in the database.
#[tokio::test]
async fn a_match_below_the_window_is_dropped_without_a_timeline_call() {
    let scenario = scenario_on_patches(&[("EUW1_21", "16.21.1.1"), ("EUW1_22", "16.22.1.1")]).await;
    store(
        &scenario,
        "seed-old",
        Game::on_patch("EUW1_20", "16.20.1.1"),
    )
    .await;

    assert_eq!(scenario.stored_ids(), ids(&["EUW1_21", "EUW1_22"]));
    assert_eq!(scenario.riot.request_count("EUW1_20/timeline"), 0);
}

// The window comes from kept matches, so a dropped game on a new patch must not shift it.
#[tokio::test]
async fn a_dropped_game_on_a_new_patch_does_not_shift_the_window() {
    let scenario = scenario_on_patches(&[("EUW1_20", "16.20.1.1"), ("EUW1_21", "16.21.1.1")]).await;
    let short = Game {
        duration_s: 60,
        ..Game::on_patch("EUW1_22", "16.22.1.1")
    };
    store(&scenario, "seed-new", short).await;
    assert_eq!(scenario.stored_ids(), ids(&["EUW1_20", "EUW1_21"]));
}
