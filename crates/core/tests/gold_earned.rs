//! The counting rules in docs/spec/mvp-core.md, as short games served over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::Snapshot;

use support::scripted_game::{ended, loading, running, updates_for, Poll, CS, LEVEL};

const LONG_SWORD: u32 = 1036;
const DAGGER: u32 = 1042;
const POTION: u32 = 2003;

async fn gold_earned(polls: Vec<Poll>) -> Vec<Option<u32>> {
    updates_for(polls)
        .await
        .into_iter()
        .map(|update| update.and_then(|snapshot| snapshot.gold_earned))
        .collect()
}

// Spending lowers currentGold, and Gold earned must not follow it down.
#[tokio::test]
async fn adds_increases_and_ignores_spending() {
    let polls = vec![
        running(1.0, 500.0, &[]),
        running(30.0, 150.0, &[(LONG_SWORD, 1)]),
        running(70.0, 171.0, &[(LONG_SWORD, 1)]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500), Some(521)]);
}

// Rounding each small tick away would lose most of the passive income.
#[tokio::test]
async fn adds_up_fractions_of_gold() {
    let polls = vec![
        running(1.0, 500.0, &[]),
        running(2.0, 500.6, &[]),
        running(3.0, 501.2, &[]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500), Some(501)]);
}

// A sale refunds gold nobody earned, and later income must count from the refund.
#[tokio::test]
async fn a_sale_is_not_counted() {
    let polls = vec![
        running(1.0, 100.0, &[(LONG_SWORD, 1)]),
        running(2.0, 345.0, &[]),
        running(3.0, 366.0, &[]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500), Some(521)]);
}

// An undo hands the price back, which is gold nobody earned.
#[tokio::test]
async fn an_undone_purchase_is_not_counted() {
    let polls = vec![
        running(1.0, 500.0, &[]),
        running(2.0, 150.0, &[(LONG_SWORD, 1)]),
        running(3.0, 500.0, &[]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500), Some(500)]);
}

// Two of an item are two entries, and selling one leaves the id in the list.
#[tokio::test]
async fn selling_one_of_two_copies_is_not_counted() {
    let polls = vec![
        running(1.0, 100.0, &[(LONG_SWORD, 1), (LONG_SWORD, 1)]),
        running(2.0, 345.0, &[(LONG_SWORD, 1)]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500)]);
}

// A stack stays one entry, so only its count shows the sale.
#[tokio::test]
async fn selling_one_of_a_stack_is_not_counted() {
    let polls = vec![
        running(1.0, 100.0, &[(POTION, 3)]),
        running(2.0, 120.0, &[(POTION, 2)]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(500)]);
}

// Gold from a kill must still count in the poll that reorders the inventory.
#[tokio::test]
async fn reordering_items_is_not_an_item_leaving() {
    let polls = vec![
        running(1.0, 100.0, &[(LONG_SWORD, 1), (DAGGER, 1)]),
        running(2.0, 400.0, &[(DAGGER, 1), (LONG_SWORD, 1)]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(800)]);
}

// A purchase can come before the first poll, and starting gold is still all earned.
#[tokio::test]
async fn a_first_snapshot_inside_the_first_minute_counts_from_500() {
    let polls = vec![
        running(59.9, 50.0, &[(LONG_SWORD, 1)]),
        running(60.9, 71.0, &[(LONG_SWORD, 1)]),
    ];
    assert_eq!(gold_earned(polls).await, [Some(500), Some(521)]);
}

// The dash takes only gold away: CS and level stay on screen for a late start.
#[tokio::test]
async fn a_later_first_snapshot_shows_a_dash_for_the_whole_game() {
    let updates = updates_for(vec![running(60.0, 500.0, &[]), running(61.0, 600.0, &[])]).await;
    let dash = || {
        Some(Snapshot {
            cs: CS,
            level: LEVEL,
            gold_earned: None,
        })
    };
    assert_eq!(updates, [dash(), dash()]);
}

// A timed-out poll has no snapshot. Restarting on it would turn the game into a dash.
#[tokio::test]
async fn a_dropped_poll_keeps_the_count() {
    let polls = vec![
        running(10.0, 500.0, &[]),
        running(100.0, 520.0, &[]),
        Poll::Dropped,
        running(102.0, 560.0, &[]),
    ];
    assert_eq!(
        gold_earned(polls).await,
        [Some(500), Some(520), None, Some(560)]
    );
}

// The loading screen reads gold 0, then 500 at GameStart, which nobody earned.
#[tokio::test]
async fn the_loading_screen_is_not_counted() {
    let polls = vec![loading(0.02), running(0.1, 500.0, &[])];
    assert_eq!(gold_earned(polls).await, [None, Some(500)]);
}

// GameEnd closes the count, so the next game starts over from 500.
#[tokio::test]
async fn a_new_game_counts_from_500_again() {
    let polls = vec![
        running(10.0, 500.0, &[]),
        running(1800.0, 900.0, &[]),
        ended(1801.0, 900.0),
        running(0.1, 500.0, &[]),
    ];
    assert_eq!(
        gold_earned(polls).await,
        [Some(500), Some(900), None, Some(500)]
    );
}

// A game joined late must not leave its dash on the next one.
#[tokio::test]
async fn a_dash_ends_with_its_game() {
    let polls = vec![
        running(600.0, 500.0, &[]),
        loading(0.02),
        running(0.1, 500.0, &[]),
    ];
    assert_eq!(gold_earned(polls).await, [None, None, Some(500)]);
}

// The connection can drop before a poll sees GameEnd or the next loading screen.
#[tokio::test]
async fn a_clock_that_turns_back_is_a_new_game() {
    let polls = vec![
        running(10.0, 500.0, &[]),
        running(900.0, 2000.0, &[]),
        Poll::Dropped,
        running(5.0, 500.0, &[]),
    ];
    assert_eq!(
        gold_earned(polls).await,
        [Some(500), Some(2000), None, Some(500)]
    );
}
