//! Tables downloaded from a fake public bucket, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::delivery::{ManifestKey, TableChannel};
use lollipop_core::live::{status_line, Markers};

use support::delivery_bench::{table_with_cs, until, DeliveryBench};
use support::fake_bucket::{manifest_body, message, sha256_of, sign, sign_with_this_bucket};
use support::fake_lcu::SESSION;
use support::staged_game::Stage;

const IN_GAME: Stage = Stage::Running { game_time: 100.0 };

#[tokio::test]
async fn a_launch_with_no_table_installs_the_one_the_manifest_names() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let table = table_with_cs(100.0);
    bench.bucket.publish(3, &table);

    bench.start_polling();
    bench.start_downloads();

    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}

// The bucket is public, and a credential sent to it would only be a secret given away.
#[tokio::test]
async fn tables_are_read_from_the_schemas_path_with_no_credentials() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let table = table_with_cs(100.0);
    bench.bucket.publish(1, &table);

    bench.start_polling();
    bench.start_downloads();
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;

    assert!(!bench.bucket.saw_credentials());
}

#[tokio::test]
async fn a_table_that_fails_its_checksum_keeps_the_previous_file() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.upload(2, &table_with_cs(300.0));
    bench.bucket.point_manifest_at(2, &table_with_cs(200.0));

    bench.start_polling();
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.live(), Some(previous));
}

#[tokio::test]
async fn a_download_cut_short_keeps_the_previous_file() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.publish_cut_short(2, &table_with_cs(200.0));

    bench.start_polling();
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.live(), Some(previous));
}

// A checksum proves the bytes are the ones published, not that this app can read them.
#[tokio::test]
async fn a_table_that_does_not_read_keeps_the_previous_file() {
    for unreadable in ["not a table", r#"{ "schema_version": 3, "cells": [] }"#] {
        let mut bench = DeliveryBench::at(Stage::Unanswered).await;
        let previous = table_with_cs(100.0);
        bench.write_live(&previous);
        bench.bucket.publish(2, unreadable);

        bench.start_polling();
        bench.start_downloads();
        until("the table is fetched", || {
            bench.bucket.table_downloads(2) == 1
        })
        .await;
        bench.settle().await;

        assert_eq!(bench.live(), Some(previous), "{unreadable}");
    }
}

#[tokio::test]
async fn a_manifest_that_does_not_read_keeps_the_previous_file() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.upload(2, &table_with_cs(200.0));
    bench.bucket.write_manifest("not a manifest");

    bench.start_polling();
    bench.start_downloads();
    until("the manifest is fetched", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.bucket.table_downloads(2), 0);
    assert_eq!(bench.live(), Some(previous));
}

// The channel is public, and anyone able to write to it must not be able to choose the table.
#[tokio::test]
async fn a_manifest_without_a_valid_signature_keeps_the_previous_file() {
    let table = table_with_cs(200.0);
    let sha256 = sha256_of(&table);
    let genuine = message(2, 2, &sha256);
    let not_signed = serde_json::json!({ "version": 2, "sha256": sha256 }).to_string();
    let forged = [
        (
            "another key",
            manifest_body(2, &sha256, &sign(&[9; 32], &genuine)),
        ),
        ("no signature", not_signed),
        ("not hex", manifest_body(2, &sha256, "zz")),
        (
            "another table",
            manifest_body(2, &sha256, &sign_with_this_bucket(&message(2, 2, "abc"))),
        ),
        (
            "another version",
            manifest_body(2, &sha256, &sign_with_this_bucket(&message(2, 3, &sha256))),
        ),
        (
            "another schema",
            manifest_body(2, &sha256, &sign_with_this_bucket(&message(3, 2, &sha256))),
        ),
    ];
    for (case, manifest) in forged {
        let mut bench = DeliveryBench::at(Stage::Unanswered).await;
        let previous = table_with_cs(100.0);
        bench.write_live(&previous);
        bench.bucket.upload(2, &table);
        bench.bucket.write_manifest(&manifest);

        bench.start_polling();
        bench.start_downloads();
        until("the manifest is fetched", || {
            bench.bucket.manifest_downloads() == 1
        })
        .await;
        bench.settle().await;

        assert_eq!(bench.bucket.table_downloads(2), 0, "{case}");
        assert_eq!(bench.live(), Some(previous), "{case}");
    }
}

// A constant that does not decode would mean no downloads at all, with nothing to show for it.
#[tokio::test]
async fn the_compiled_in_key_decodes_and_refuses_a_table_it_did_not_sign() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.publish(2, &table_with_cs(200.0));

    bench.start_polling();
    bench.start_downloads_with(bench.bucket.channel(), ManifestKey::compiled_in().unwrap());
    until("the manifest is fetched", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.bucket.table_downloads(2), 0);
    assert_eq!(bench.live(), Some(previous));
}

// A bucket with no manifest yet answers 404, and that is not a reason to touch anything.
#[tokio::test]
async fn a_missing_manifest_keeps_the_previous_file() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);

    bench.start_polling();
    bench.start_downloads();
    until("the manifest is asked for", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.live(), Some(previous));
}

#[tokio::test]
async fn an_outage_keeps_the_previous_file() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.publish(2, &table_with_cs(200.0));
    bench.bucket.go_down();

    bench.start_polling();
    bench.start_downloads();
    bench.settle().await;

    assert_eq!(bench.live(), Some(previous));
}

// One failed check must not strand the app on an old table: the next game's check retries.
#[tokio::test]
async fn a_failed_download_is_tried_again_after_the_next_game() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    bench.write_live(&table_with_cs(100.0));
    bench.bucket.publish_cut_short(2, &table_with_cs(200.0));

    bench.start_polling();
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    let table = table_with_cs(200.0);
    bench.bucket.publish(2, &table);
    bench.game.move_to(Stage::Ended);

    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
    assert_eq!(bench.bucket.table_downloads(2), 2);
}

#[tokio::test]
async fn a_table_already_live_is_not_downloaded_again() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let table = table_with_cs(100.0);
    bench.write_live(&table);
    bench.bucket.publish(2, &table);

    bench.start_polling();
    bench.start_downloads();
    until("the manifest is fetched", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.bucket.table_downloads(2), 0);
}

// ADR 0003. A restart mid-game reads the file again, so the file itself must hold still.
// Queue 3140 hides the Markers, and that must not open the door.
#[tokio::test]
async fn a_table_is_never_swapped_during_a_game_even_with_markers_hidden() {
    for queue in [420, 3140] {
        let mut bench = DeliveryBench::at(IN_GAME).await;
        let session = format!(r#"{{ "gameData": {{ "queue": {{ "id": {queue} }} }} }}"#);
        bench.lcu.set(SESSION, &session);
        let previous = table_with_cs(100.0);
        let table = table_with_cs(200.0);
        bench.write_live(&previous);
        bench.bucket.publish(2, &table);

        bench.start_polling();
        let game = bench.next_game_update().await;
        assert_eq!(game.markers == Markers::default(), queue == 3140);
        bench.start_downloads();
        until("the table is fetched", || {
            bench.bucket.table_downloads(2) == 1
        })
        .await;
        bench.settle().await;
        assert_eq!(bench.live().as_ref(), Some(&previous), "queue {queue}");

        bench.game.move_to(Stage::Ended);
        until("the table is live", || {
            bench.live().as_ref() == Some(&table)
        })
        .await;
    }
}

// A game with no table at all has nothing to lose, and is still a game.
#[tokio::test]
async fn a_first_table_also_waits_for_the_game_to_end() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let table = table_with_cs(100.0);
    bench.bucket.publish(1, &table);

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(1) == 1
    })
    .await;
    bench.settle().await;
    assert_eq!(bench.live(), None);

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}

// League stops answering for a moment mid-game, and that is not the end of the game.
#[tokio::test]
async fn a_failed_poll_during_a_game_does_not_open_the_swap() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let previous = table_with_cs(100.0);
    let table = table_with_cs(200.0);
    bench.write_live(&previous);
    bench.bucket.publish(2, &table);

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;

    bench.game.move_to(Stage::Unanswered);
    bench.settle().await;
    assert_eq!(bench.live().as_ref(), Some(&previous));

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}

// A running match that does not list the player yet holds no game, and is still a game.
#[tokio::test]
async fn a_running_match_with_no_active_player_does_not_open_the_swap() {
    let mut bench = DeliveryBench::at(Stage::RunningWithNoActivePlayer).await;
    let previous = table_with_cs(100.0);
    let table = table_with_cs(200.0);
    bench.write_live(&previous);
    bench.bucket.publish(2, &table);

    bench.start_polling();
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;
    assert_eq!(bench.live().as_ref(), Some(&previous));

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}

#[tokio::test]
async fn the_game_after_a_swap_reads_the_new_table_and_the_one_before_it_keeps_the_old() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let no_cell_for_this_player = support::benchmark_file::table_of(&[]);
    bench.write_live(&table_with_cs(100.0));
    bench.bucket.publish(2, &no_cell_for_this_player);

    bench.start_polling();
    assert_ne!(bench.next_game_update().await.markers, Markers::default());
    bench.start_downloads();
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;
    assert_ne!(bench.next_game_update().await.markers, Markers::default());

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&no_cell_for_this_player)
    })
    .await;
    bench.game.move_to(IN_GAME);
    bench.settle().await;
    assert_eq!(bench.next_game_update().await.markers, Markers::default());
}

#[tokio::test]
async fn each_finished_game_checks_the_manifest_once() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    bench.write_live(&table_with_cs(100.0));
    bench.bucket.publish(1, &table_with_cs(100.0));

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the launch check is done", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    let table = table_with_cs(200.0);
    bench.bucket.publish(2, &table);
    bench.settle().await;
    assert_eq!(bench.bucket.manifest_downloads(), 1);

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
    bench.settle().await;
    assert_eq!(bench.bucket.manifest_downloads(), 2);

    bench.game.move_to(IN_GAME);
    bench.settle().await;
    assert_eq!(bench.bucket.manifest_downloads(), 2);
    bench.game.move_to(Stage::Ended);
    until("the second game's check is done", || {
        bench.bucket.manifest_downloads() == 3
    })
    .await;
}

// The connection can drop before a poll sees GameEnd, and the clock then shows the new game.
// The check still runs, and what it finds waits for this new game to end.
#[tokio::test]
async fn a_clock_turned_back_counts_as_a_finished_game() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let previous = table_with_cs(100.0);
    bench.write_live(&previous);
    bench.bucket.publish(1, &previous);

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the launch check is done", || {
        bench.bucket.manifest_downloads() == 1
    })
    .await;
    let table = table_with_cs(200.0);
    bench.bucket.publish(2, &table);

    bench.game.move_to(Stage::Running { game_time: 5.0 });
    until("the table is fetched", || {
        bench.bucket.table_downloads(2) == 1
    })
    .await;
    bench.settle().await;
    assert_eq!(bench.live().as_ref(), Some(&previous));

    bench.game.move_to(Stage::Ended);
    until("the table is live", || {
        bench.live().as_ref() == Some(&table)
    })
    .await;
}

// A rollback: `lollipop-collector rollback` points the manifest at a lower version.
#[tokio::test]
async fn the_app_follows_the_manifest_to_an_older_version() {
    let mut bench = DeliveryBench::at(Stage::Unanswered).await;
    let older = table_with_cs(100.0);
    bench.write_live(&table_with_cs(200.0));
    bench.bucket.upload(5, &table_with_cs(200.0));
    bench.bucket.publish(4, &older);

    bench.start_polling();
    bench.start_downloads();

    until("the older table is live", || {
        bench.live().as_ref() == Some(&older)
    })
    .await;
}

// The rollback lands while the bad table is staged, waiting out a game. It must not stay.
#[tokio::test]
async fn a_rollback_beats_a_table_still_waiting_for_its_swap() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let older = table_with_cs(100.0);
    bench.write_live(&older);
    bench.bucket.upload(4, &older);
    bench.bucket.publish(5, &table_with_cs(200.0));

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the bad table is fetched", || {
        bench.bucket.table_downloads(5) == 1
    })
    .await;
    bench.settle().await;
    bench.bucket.point_manifest_at(4, &older);

    bench.game.move_to(Stage::Ended);
    until("the check after the game is done", || {
        bench.bucket.manifest_downloads() == 2
    })
    .await;

    until("the older table is live", || {
        bench.live().as_ref() == Some(&older)
    })
    .await;
}

// A check made during the next game finds the rollback already live: the bad table must
// not go live when that game ends, not even for the moment before a repair.
#[tokio::test]
async fn a_table_the_manifest_stopped_naming_is_not_swapped_in() {
    let mut bench = DeliveryBench::at(IN_GAME).await;
    let older = table_with_cs(100.0);
    bench.write_live(&older);
    bench.bucket.upload(4, &older);
    bench.bucket.publish(5, &table_with_cs(200.0));

    bench.start_polling();
    bench.next_game_update().await;
    bench.start_downloads();
    until("the bad table is fetched", || {
        bench.bucket.table_downloads(5) == 1
    })
    .await;
    bench.settle().await;
    bench.bucket.point_manifest_at(4, &older);

    bench.game.move_to(Stage::Running { game_time: 5.0 });
    until("the next game's check is done", || {
        bench.bucket.manifest_downloads() == 2
    })
    .await;
    bench.settle().await;
    bench.game.move_to(Stage::Ended);
    until("the check after that game is done", || {
        bench.bucket.manifest_downloads() == 3
    })
    .await;
    bench.settle().await;

    assert_eq!(bench.live().as_ref(), Some(&older));
    assert_eq!(bench.bucket.table_downloads(4), 0);
}

#[tokio::test]
async fn with_no_table_and_no_network_only_the_markers_are_missing() {
    // 4:30 is inside the first drake's hint.
    let mut bench = DeliveryBench::at(Stage::Running { game_time: 270.0 }).await;
    // Nothing listens on port 1.
    let channel = TableChannel::new("http://127.0.0.1:1").unwrap();

    bench.start_polling();
    bench.start_downloads_on(channel);
    let game = bench.next_game_update().await;
    bench.settle().await;

    assert_eq!(game.markers, Markers::default());
    assert_eq!(game.snapshot.cs, 95);
    assert_eq!(game.snapshot.level, 10);
    assert_eq!(
        status_line(Some(&game)),
        "Master Yi · Mid · Gold · No Benchmarks yet"
    );
    assert!(!game.hints.is_empty());
    assert_eq!(bench.live(), None);
}
