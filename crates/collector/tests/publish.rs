//! Publishing through the built binary, against a fake `gh` holding the Release.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::process::Output;

use lollipop_core::benchmarks::BenchmarkTable;
use lollipop_core::game_inputs::{Role, Tier};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use support::played::Played;
use support::release_bench::{cell, table_of, ReleaseBench};
use support::table_bench::TableBench;

const WINDOW: [&str; 2] = ["16.20", "16.21"];

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn assert_published(output: &Output) {
    assert!(output.status.success(), "{}", stderr(output));
}

fn bench_on_version_1(cells: usize) -> ReleaseBench {
    let mut bench = ReleaseBench::new();
    assert_published(&bench.publish_table(&table_of(cells, &WINDOW)));
    bench
}

/// Publishes, and checks the old table stays live with an issue opened about it.
fn assert_refused(bench: &mut ReleaseBench, table: &Value) {
    let live = bench.manifest_version();
    let assets = bench.asset_names();
    let output = bench.publish_table(table);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("refused"), "{}", stderr(&output));
    assert_eq!(bench.manifest_version(), live);
    assert_eq!(bench.asset_names(), assets);
    assert!(bench
        .issues()
        .last()
        .unwrap()
        .contains("Benchmark table publish refused"));
}

// The app builds the path from this: `v1/manifest.json` next to the table, whose sha256
// it checks before a swap. A wrong hash would make it throw away every download.
#[test]
fn a_first_publish_creates_the_release_with_a_table_and_its_manifest() {
    let mut bench = ReleaseBench::new();
    assert_published(&bench.publish_table(&table_of(3, &WINDOW)));

    assert_eq!(bench.tag(), "v1");
    assert_eq!(
        bench.asset_names(),
        ["benchmark-table-1.json", "manifest.json"]
    );
    let body = std::fs::read(bench.asset_path("benchmark-table-1.json")).unwrap();
    assert_eq!(
        bench.asset("manifest.json"),
        json!({"version": 1, "sha256": format!("{:x}", Sha256::digest(body))})
    );
    assert!(bench.issues().is_empty());
}

// The whole path from a database, read back the way the app reads it.
#[test]
fn a_built_table_is_published_and_its_drift_report_printed() {
    let mut matches = TableBench::new();
    matches.add(100, &Played::new("Ahri", "GOLD"));
    let bench = ReleaseBench::new();

    let output = bench.publish_built(&matches.database(), &[]);

    assert_published(&output);
    assert!(stdout(&output).contains("drift report"));
    assert!(stdout(&output).contains("published table 1"));
    let table = BenchmarkTable::read(&bench.asset_path("benchmark-table-1.json")).unwrap();
    assert!(table.into_cell("Ahri", Role::Top, Tier::Gold).is_some());
}

#[test]
fn a_drop_of_more_than_20_percent_of_cells_leaves_the_old_table() {
    let mut bench = bench_on_version_1(9);
    assert_refused(&mut bench, &table_of(7, &WINDOW));
    assert!(bench.issues()[0].contains("2 of 9 Cells"));
}

// The gate is "more than" 20%, and a provisional number should stay exact until rechecked.
#[test]
fn a_drop_of_exactly_20_percent_publishes() {
    let mut bench = bench_on_version_1(5);
    assert_published(&bench.publish_table(&table_of(4, &WINDOW)));
    assert_eq!(bench.manifest_version(), 2);
}

// A count would call this no drop at all, while most players would lose their Marker.
#[test]
fn new_cells_do_not_make_up_for_the_ones_lost() {
    let mut bench = bench_on_version_1(5);
    let other_champions: Vec<Value> = (0..5).map(|n| cell(&format!("Other{n}"))).collect();
    let mut table = table_of(0, &WINDOW);
    table["cells"] = json!(other_champions);
    assert_refused(&mut bench, &table);
}

// A rework is added before the patch ships, and takes all of that champion's Cells with it.
#[test]
fn a_rework_entry_does_not_count_as_a_drop() {
    let mut bench = bench_on_version_1(3);
    bench.set_reworks(
        r#"[{"champion": "Champion1", "first_valid_patch": "16.21"},
        {"champion": "Champion2", "first_valid_patch": "16.21"}]"#,
    );
    let mut table = table_of(1, &WINDOW);
    table["patch_window"] = json!(["16.21", "16.22"]);
    assert_published(&bench.publish_table(&table));
    assert_eq!(bench.manifest_version(), 2);
}

// An entry stays in the file for good, and must not excuse that champion's losses forever.
#[test]
fn a_rework_older_than_the_old_table_still_counts_as_a_drop() {
    let mut bench = bench_on_version_1(3);
    bench.set_reworks(
        r#"[{"champion": "Champion1", "first_valid_patch": "16.20"},
        {"champion": "Champion2", "first_valid_patch": "16.20"}]"#,
    );
    assert_refused(&mut bench, &table_of(1, &WINDOW));
}

fn table_with_max_patch(cells: usize, max_patch: &str) -> Value {
    let mut table = table_of(cells, &["16.23", "16.24"]);
    table["max_patch"] = json!(max_patch);
    table
}

// A season patch resets the economy, so the first table on its data is thin by design.
#[test]
fn data_past_a_max_patch_may_drop_any_number_of_cells() {
    let mut bench = ReleaseBench::new();
    assert_published(&bench.publish_table(&table_with_max_patch(10, "16.24")));
    assert_published(&bench.publish_table(&table_of(1, &["16.24", "17.1"])));
    assert_eq!(bench.manifest_version(), 2);
    assert_eq!(
        bench.asset("benchmark-table-2.json")["cells"],
        json!([cell("Champion0")])
    );
}

// Only new data lifts it: otherwise setting `max_patch` would disable the gate at once.
#[test]
fn a_max_patch_with_no_data_past_it_keeps_the_gate() {
    let mut bench = ReleaseBench::new();
    assert_published(&bench.publish_table(&table_with_max_patch(10, "16.24")));
    assert_refused(&mut bench, &table_of(1, &["16.23", "16.24"]));
}

fn matches_on(patch: (u32, u32)) -> TableBench {
    let mut matches = TableBench::new();
    let played = Played {
        patch,
        ..Played::new("Ahri", "GOLD")
    };
    matches.add(100, &played);
    matches
}

// The weekly build passes no flag, and `max_patch` guards a season patch not yet here.
#[test]
fn a_build_keeps_the_max_patch_until_data_passes_it() {
    let bench = ReleaseBench::new();
    let on_old_data = matches_on((16, 24));
    assert_published(&bench.publish_built(&on_old_data.database(), &["--max-patch", "16.24"]));
    let mut more = matches_on((16, 24));
    more.add(
        1,
        &Played {
            patch: (16, 23),
            ..Played::new("Ahri", "GOLD")
        },
    );

    assert_published(&bench.publish_built(&more.database(), &[]));
    assert_eq!(
        bench.asset("benchmark-table-2.json")["max_patch"],
        json!("16.24")
    );

    assert_published(&bench.publish_built(&matches_on((17, 1)).database(), &[]));
    assert_eq!(bench.asset("benchmark-table-3.json").get("max_patch"), None);
}

#[test]
fn a_file_that_is_not_json_leaves_the_old_table() {
    let mut bench = bench_on_version_1(3);
    let output = bench.publish_file("{ not json");
    assert!(!output.status.success());
    assert_eq!(bench.manifest_version(), 1);
    assert_eq!(bench.asset_names().len(), 2);
    assert!(bench.issues()[0].contains("does not read"));
}

// Valid JSON the app would still refuse: publishing it would cost every install its Markers.
#[test]
fn a_file_the_app_cannot_read_leaves_the_old_table() {
    let mut bench = bench_on_version_1(3);
    let mut other_schema = table_of(3, &WINDOW);
    other_schema["schema_version"] = json!(2);
    assert_refused(&mut bench, &other_schema);
}

// A first publish has no old table to protect, but must not ship a broken file either.
#[test]
fn a_file_that_does_not_read_is_refused_on_a_first_publish() {
    let mut bench = ReleaseBench::new();
    assert!(!bench.publish_file("{ not json").status.success());
    assert!(bench.asset_names().is_empty());
    assert_eq!(bench.issues().len(), 1);
}

#[test]
fn a_failed_build_opens_an_issue_and_leaves_the_old_table() {
    let bench = bench_on_version_1(3);
    bench.set_reworks("not json");
    let matches = matches_on((16, 21));

    let output = bench.publish_built(&matches.database(), &[]);

    assert!(!output.status.success());
    assert_eq!(bench.manifest_version(), 1);
    assert!(bench.issues()[0].contains("Benchmark table build failed"));
}

#[test]
fn an_empty_database_opens_an_issue() {
    let bench = bench_on_version_1(3);
    let output = bench.publish_built(&TableBench::new().database(), &[]);
    assert!(!output.status.success());
    assert!(bench.issues()[0].contains("no matches"));
}

// The notice is the only signal a cron run has, and a failed upload must not pass silently.
#[test]
fn a_failed_upload_opens_an_issue_and_leaves_the_manifest() {
    let mut bench = bench_on_version_1(3);
    bench.lose_upload("benchmark-table-2.json");
    let output = bench.publish_table(&table_of(4, &WINDOW));
    assert!(!output.status.success());
    assert_eq!(bench.manifest_version(), 1);
    assert!(bench.issues()[0].contains("Benchmark table publish failed"));
}

// Without a working `gh` the run can only say so, and that must name both failures.
#[test]
fn a_notice_that_fails_is_reported_with_its_cause() {
    let mut bench = bench_on_version_1(3);
    bench.break_gh("issue-create");
    let output = bench.publish_file("{ not json");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("does not read"));
    assert!(stderr(&output).contains("did not open"));
}

// An outage looks like "no release" only by its message: taking it for a first publish
// would skip the gate.
#[test]
fn an_outage_reading_the_release_publishes_nothing() {
    let mut bench = bench_on_version_1(3);
    bench.break_gh("release-view");
    let output = bench.publish_table(&table_of(1, &WINDOW));
    assert!(!output.status.success());
    bench.heal();
    assert_eq!(bench.manifest_version(), 1);
    assert_eq!(bench.asset_names().len(), 2);
    assert_eq!(bench.issues().len(), 1);
}

// `--clobber` deletes before it uploads. A run that dies there must not make the next one
// a gateless first publish.
#[test]
fn a_manifest_lost_in_upload_stops_publishing_until_a_rollback() {
    let mut bench = bench_on_version_1(5);
    bench.lose_upload("manifest.json");
    assert!(!bench
        .publish_table(&table_of(5, &["16.21", "16.22"]))
        .status
        .success());
    bench.heal();
    assert!(!bench.asset_names().contains(&"manifest.json".to_string()));

    let refused = bench.publish_table(&table_of(1, &WINDOW));
    assert!(!refused.status.success());
    assert!(stderr(&refused).contains("no manifest"));
    assert!(!bench.asset_names().contains(&"manifest.json".to_string()));

    assert!(bench.rollback(1).status.success());
    assert_eq!(bench.manifest_version(), 1);
    assert_refused(&mut bench, &table_of(1, &WINDOW));
}

#[test]
fn an_unreadable_old_table_refuses_until_a_rollback() {
    let mut bench = bench_on_version_1(3);
    assert_published(&bench.publish_table(&table_of(4, &WINDOW)));
    bench.overwrite_asset("benchmark-table-2.json", "{ not json");

    let output = bench.publish_table(&table_of(4, &["16.21", "16.22"]));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("rollback"));

    assert!(bench.rollback(1).status.success());
    assert_published(&bench.publish_table(&table_of(3, &["16.21", "16.22"])));
}

#[test]
fn a_manifest_that_does_not_read_or_names_a_lost_table_refuses() {
    let mut bench = bench_on_version_1(3);
    bench.overwrite_asset("manifest.json", "{ not json");
    assert!(stderr(&bench.publish_table(&table_of(4, &WINDOW))).contains("manifest"));

    bench.overwrite_asset("manifest.json", r#"{"version": 9, "sha256": "x"}"#);
    assert!(stderr(&bench.publish_table(&table_of(4, &WINDOW))).contains("table 9"));
    assert_eq!(bench.issues().len(), 2);
}

fn publish_versions(bench: &mut ReleaseBench, count: usize) {
    for version in 0..count {
        assert_published(&bench.publish_table(&table_of(10 + version, &WINDOW)));
    }
}

#[test]
fn only_the_last_4_tables_are_kept() {
    let mut bench = ReleaseBench::new();
    publish_versions(&mut bench, 6);
    assert_eq!(
        bench.asset_names(),
        [
            "benchmark-table-3.json",
            "benchmark-table-4.json",
            "benchmark-table-5.json",
            "benchmark-table-6.json",
            "manifest.json",
        ]
    );
    assert_eq!(bench.manifest_version(), 6);
}

// With the collector down, identical weekly builds would push every rollback target out.
#[test]
fn an_identical_table_takes_no_slot() {
    let mut bench = bench_on_version_1(3);
    let output = bench.publish_table(&table_of(3, &WINDOW));
    assert_published(&output);
    assert!(stdout(&output).contains("table 1 is unchanged"));
    assert_eq!(bench.asset_names().len(), 2);
}

#[test]
fn a_rollback_points_the_manifest_at_an_older_table_with_its_hash() {
    let mut bench = ReleaseBench::new();
    publish_versions(&mut bench, 3);
    let before = bench.asset_names();

    assert!(bench.rollback(1).status.success());

    assert_eq!(bench.asset_names(), before);
    let body = std::fs::read(bench.asset_path("benchmark-table-1.json")).unwrap();
    assert_eq!(
        bench.asset("manifest.json"),
        json!({"version": 1, "sha256": format!("{:x}", Sha256::digest(body))})
    );
}

// The manifest is the truth: the gate reads the table it names, not the newest one. A
// reused version number would hand the app a different file under a name it has seen.
#[test]
fn a_publish_after_a_rollback_compares_to_the_live_table_and_takes_a_new_version() {
    let mut bench = ReleaseBench::new();
    assert_published(&bench.publish_table(&table_of(5, &WINDOW)));
    assert_published(&bench.publish_table(&table_of(10, &WINDOW)));
    assert!(bench.rollback(1).status.success());

    assert_published(&bench.publish_table(&table_of(4, &WINDOW)));

    assert_eq!(bench.manifest_version(), 3);
}

#[test]
fn a_rollback_to_a_table_not_kept_leaves_the_manifest() {
    let mut bench = ReleaseBench::new();
    publish_versions(&mut bench, 6);
    let output = bench.rollback(1);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("table 1 is not kept"));
    assert_eq!(bench.manifest_version(), 6);
    assert!(bench.issues().is_empty());
}

#[test]
fn a_rollback_to_a_table_that_does_not_read_leaves_the_manifest() {
    let mut bench = ReleaseBench::new();
    publish_versions(&mut bench, 2);
    bench.overwrite_asset("benchmark-table-1.json", "{ not json");
    assert!(!bench.rollback(1).status.success());
    assert_eq!(bench.manifest_version(), 2);
}
