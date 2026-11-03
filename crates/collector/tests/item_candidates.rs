//! Item candidates and the Damage profile in the Benchmark table, built by the binary
//! from archived matches and Data Dragon's item classes.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;

use lollipop_collector::SeedTier;
use lollipop_core::benchmarks::BenchmarkTable;
use serde_json::json;

use support::archived_match::{damage, ArchivedMatch, Seat, FILLER};
use support::build::{run_build, Build, Candidate};
use support::fake_data_dragon::{
    FakeDataDragon, ARCHANGELS_STAFF, ARENA_ITEM, BF_SWORD, BOOTS, COSTLY_COMPONENT, DEMOTED_ITEM,
    DORANS_BLADE, HEALTH_POTION, INFINITY_EDGE, LONG_SWORD, MANAMUNE, MERCURYS_TREADS, MORE_BOOTS,
    MORE_COMPLETED, MURAMANA, PLATED_STEELCAPS, QUEST_ITEM, QUEST_ITEM_STEP_1, QUEST_ITEM_STEP_2,
    RABADONS, REMOVED_ITEM, SERAPHS_EMBRACE, TIER_3_BOOTS, ZHONYAS,
};
use support::game::Game;
use support::played::Played;
use support::scenario::Scenario;
use support::table_bench::TableBench;

const BUCKETS: [&str; 3] = ["physical_heavy", "mixed", "magic_heavy"];

const AHRI: Seat = Seat {
    champion: "Ahri",
    role: "MIDDLE",
    magic: 9000,
    total: 10_000,
};
const DARIUS: Seat = Seat {
    champion: "Darius",
    role: "TOP",
    magic: 0,
    total: 10_000,
};

fn ahri_match() -> ArchivedMatch {
    ArchivedMatch::new().seat(1, AHRI)
}

fn min_games(count: u32) -> [String; 2] {
    ["--min-candidate-games".to_string(), count.to_string()]
}

fn build(bench: &TableBench, min: u32) -> Build {
    let args = min_games(min);
    bench.build(&[&args[0], &args[1]])
}

/// The one list of a champion and role whose games all faced the same bucket, so every
/// other bucket took the pooled list.
fn only_list(build: &Build, champion: &str, role: &str) -> Vec<Candidate> {
    let lists: Vec<Vec<Candidate>> = BUCKETS
        .iter()
        .map(|bucket| build.candidates(champion, role, bucket).unwrap())
        .collect();
    assert!(lists.windows(2).all(|pair| pair[0] == pair[1]), "{lists:?}");
    lists[0].clone()
}

fn items(list: &[Candidate]) -> Vec<u32> {
    list.iter().map(|candidate| candidate.0).collect()
}

// The app hands the model this list: a component, a potion or an Arena item in it would
// be advice nobody on the Rift can follow.
#[test]
fn candidates_are_completed_items_and_tier_2_boots_with_their_share_and_minute() {
    let mut bench = TableBench::new();
    let full_build = ahri_match()
        .buys(1, DORANS_BLADE, 0)
        .buys(1, HEALTH_POTION, 1)
        .buys(1, BOOTS, 3)
        .buys(1, MERCURYS_TREADS, 8)
        .buys(1, RABADONS, 12)
        .buys(1, COSTLY_COMPONENT, 15)
        .buys(1, ARENA_ITEM, 20)
        .buys(1, ZHONYAS, 22);
    bench.add_archived(3, &full_build);
    bench.add_archived(1, &ahri_match().buys(1, RABADONS, 14));

    let build = build(&bench, 4);

    assert_eq!(
        only_list(&build, "Ahri", "MIDDLE"),
        [
            (RABADONS, 1.0, 12.0),
            (MERCURYS_TREADS, 0.75, 8.0),
            (ZHONYAS, 0.75, 22.0),
        ]
    );
}

// A purchase taken back in the shop was never bought; an undone sale leaves the purchase.
#[test]
fn an_undone_purchase_does_not_count() {
    let mut bench = TableBench::new();
    let undoing = ahri_match()
        .buys(1, ZHONYAS, 5)
        .undoes(1, ZHONYAS, 0, 5)
        .buys(1, RABADONS, 10)
        .buys(1, INFINITY_EDGE, 12)
        .undoes(1, 0, INFINITY_EDGE, 13);
    bench.add_archived(2, &undoing);

    let build = build(&bench, 2);

    assert_eq!(
        items(&only_list(&build, "Ahri", "MIDDLE")),
        [INFINITY_EDGE, RABADONS]
    );
}

// A rare item past 90% coverage is noise, and would cost the prompt tokens.
#[test]
fn the_list_stops_once_it_covers_90_percent_of_purchases() {
    let mut bench = TableBench::new();
    let all_three = ahri_match()
        .buys(1, INFINITY_EDGE, 10)
        .buys(1, RABADONS, 20)
        .buys(1, ZHONYAS, 25);
    bench.add_archived(1, &all_three);
    let two = ahri_match()
        .buys(1, INFINITY_EDGE, 10)
        .buys(1, RABADONS, 20);
    bench.add_archived(4, &two);
    bench.add_archived(5, &ahri_match().buys(1, INFINITY_EDGE, 10));

    let build = build(&bench, 10);

    let list = only_list(&build, "Ahri", "MIDDLE");
    assert_eq!(list, [(INFINITY_EDGE, 1.0, 10.0), (RABADONS, 0.5, 20.0)]);
}

#[test]
fn the_list_holds_at_most_12_completed_items_and_3_boots() {
    let mut bench = TableBench::new();
    let mut everything = ahri_match();
    for index in 0..16 {
        everything = everything.buys(1, MORE_COMPLETED + index, u64::from(index));
    }
    for index in 0..5 {
        everything = everything.buys(1, MORE_BOOTS + index, u64::from(index));
    }
    bench.add_archived(1, &everything);

    let build = build(&bench, 1);

    let mut expected: Vec<u32> = (0..12).map(|index| MORE_COMPLETED + index).collect();
    expected.extend((0..3).map(|index| MORE_BOOTS + index));
    let mut kept = items(&only_list(&build, "Ahri", "MIDDLE"));
    kept.sort_unstable();
    assert_eq!(kept, expected);
}

// The table serves the newest patch: an item that left the game, or became a component,
// bought on an older one would be advice nobody can follow.
#[test]
fn items_are_classed_by_the_newest_patchs_data_dragon_version() {
    let mut bench = TableBench::new();
    let older = ahri_match()
        .on_patch(16, 20)
        .buys(1, REMOVED_ITEM, 10)
        .buys(1, DEMOTED_ITEM, 11);
    bench.add_archived(1, &older);
    bench.add_archived(1, &ahri_match().buys(1, RABADONS, 12));

    let build = build(&bench, 2);

    assert_eq!(items(&only_list(&build, "Ahri", "MIDDLE")), [RABADONS]);
}

// The view loads icons by this version, and Data Dragon lags a new patch by days: the
// patch label would point at icons that do not exist yet.
#[test]
fn the_file_carries_the_data_dragon_version_the_items_were_read_at() {
    let mut bench = TableBench::new();
    bench.add_archived(1, &ahri_match().on_patch(16, 22).buys(1, RABADONS, 12));

    let build = build(&bench, 1);

    assert_eq!(build.table()["data_dragon_version"], json!("16.21.1"));
}

// Seraph's Embrace has no purchase event: without the mapping Archangel's Staff would
// drop out of every list, and a player holding Seraph's would be advised it again.
#[test]
fn an_item_that_transforms_is_a_candidate_owned_as_its_transforms() {
    let mut bench = TableBench::new();
    let transforming = ahri_match()
        .buys(1, ARCHANGELS_STAFF, 10)
        .buys(1, MANAMUNE, 12)
        .buys(1, QUEST_ITEM, 14)
        .buys(1, RABADONS, 16);
    bench.add_archived(1, &transforming);

    let build = build(&bench, 1);

    let owned_as = |item: u32| {
        let entry = build.table()["item_candidates"][0]["mixed"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["item"] == item)
            .cloned()
            .unwrap();
        entry.get("owned_as").cloned()
    };
    assert_eq!(owned_as(ARCHANGELS_STAFF), Some(json!([SERAPHS_EMBRACE])));
    assert_eq!(owned_as(MANAMUNE), Some(json!([MURAMANA])));
    assert_eq!(
        owned_as(QUEST_ITEM),
        Some(json!([QUEST_ITEM_STEP_1, QUEST_ITEM_STEP_2]))
    );
    assert_eq!(owned_as(RABADONS), None);
}

// A held component ties the player to the candidate it builds into, however deep it sits.
#[test]
fn a_candidate_carries_every_component_it_builds_from() {
    let mut bench = TableBench::new();
    let full_build = ahri_match()
        .buys(1, MERCURYS_TREADS, 8)
        .buys(1, INFINITY_EDGE, 12)
        .buys(1, RABADONS, 16);
    bench.add_archived(1, &full_build);

    let build = build(&bench, 1);

    let builds_from = |item: u32| {
        let entry = build.table()["item_candidates"][0]["mixed"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| candidate["item"] == item)
            .cloned()
            .unwrap();
        entry.get("builds_from").cloned()
    };
    assert_eq!(
        builds_from(INFINITY_EDGE),
        Some(json!([LONG_SWORD, BF_SWORD, COSTLY_COMPONENT]))
    );
    assert_eq!(builds_from(MERCURYS_TREADS), Some(json!([BOOTS])));
    assert_eq!(builds_from(RABADONS), None);
}

// A player holds one pair of boots: without every pair listed, the app would advise
// other boots to a player who owns a pair no candidate list names.
#[test]
fn the_file_lists_every_boots_past_the_first_tier_tagged_or_not() {
    let mut bench = TableBench::new();
    bench.add_archived(1, &ahri_match().buys(1, RABADONS, 12));

    let build = build(&bench, 1);

    let mut expected = vec![PLATED_STEELCAPS, MERCURYS_TREADS, TIER_3_BOOTS];
    expected.extend((0..5).map(|index| MORE_BOOTS + index));
    assert_eq!(build.table()["boots"], json!(expected));
}

// A player who finishes an item off the plan has moved on: without every completed id,
// the app could not tell when one no list names was finished.
#[test]
fn the_file_lists_every_completed_item_on_the_newest_patch() {
    let mut bench = TableBench::new();
    bench.add_archived(1, &ahri_match().buys(1, RABADONS, 12));

    let build = build(&bench, 1);

    let completed: Vec<u64> = build.table()["completed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_u64().unwrap())
        .collect();
    for item in [
        INFINITY_EDGE,
        RABADONS,
        ZHONYAS,
        ARCHANGELS_STAFF,
        QUEST_ITEM,
    ] {
        assert!(completed.contains(&u64::from(item)), "{item} is missing");
    }
    for item in [
        MERCURYS_TREADS,
        COSTLY_COMPONENT,
        SERAPHS_EMBRACE,
        HEALTH_POTION,
        ARENA_ITEM,
        DEMOTED_ITEM,
    ] {
        assert!(!completed.contains(&u64::from(item)), "{item} is listed");
    }
    assert!(completed.is_sorted());
}

// The prompt names items to the model in the player's language, and the file is all the
// app has: a candidate with no name could not be offered.
#[test]
fn every_candidate_and_no_other_item_has_its_en_and_ru_name() {
    let mut bench = TableBench::new();
    let full_build = ahri_match()
        .buys(1, DORANS_BLADE, 0)
        .buys(1, RABADONS, 12)
        .buys(1, ZHONYAS, 22);
    bench.add_archived(1, &full_build);

    let build = build(&bench, 1);

    assert_eq!(
        build.table()["item_names"],
        json!({
            RABADONS.to_string(): { "en": "Item 3089", "ru": "Предмет 3089" },
            // Data Dragon's ru_RU lacks it here, so the English name stands in.
            ZHONYAS.to_string(): { "en": "Item 3157", "ru": "Item 3157" },
        })
    );
}

fn team_of(archived: ArchivedMatch, team: u32, magic_seats: u32) -> ArchivedMatch {
    let first = if team == 100 { 1 } else { 6 };
    (0..5).fold(archived, |archived, seat| {
        let champion = if seat < magic_seats { "Lux" } else { FILLER };
        let magic = if seat < magic_seats { 10_000 } else { 0 };
        archived.seat(first + seat, damage(champion, magic))
    })
}

fn background(bench: &mut TableBench, blue: u32, red: u32) {
    let archived = team_of(team_of(ArchivedMatch::new(), 100, blue), 200, red);
    bench.add_archived(1, &archived);
}

/// Darius on team 100 with four physical seats, facing `magic_seats` Lux.
fn darius_facing(magic_seats: u32, item: u32) -> ArchivedMatch {
    team_of(ArchivedMatch::new(), 200, magic_seats)
        .seat(1, DARIUS)
        .buys(1, item, 9)
}

/// Team sums 0, 0, 0, 0, 1, 1, 2, 3, 3, 4, 5, 5: bounds 0.667 and 3. Darius faces 0, 2
/// and 5, one game in each bucket.
fn bench_with_one_darius_game_per_bucket() -> TableBench {
    let mut bench = TableBench::new();
    background(&mut bench, 1, 1);
    background(&mut bench, 3, 3);
    background(&mut bench, 4, 5);
    bench.add_archived(1, &darius_facing(0, PLATED_STEELCAPS));
    bench.add_archived(1, &darius_facing(2, INFINITY_EDGE));
    bench.add_archived(1, &darius_facing(5, MERCURYS_TREADS));
    bench
}

// The app sums the five enemies' shares and compares that to the bounds: both ends must
// agree on what a share and a bound mean.
#[test]
fn the_damage_profile_holds_each_champions_magic_share_and_tercile_bounds() {
    let build = build(&bench_with_one_darius_game_per_bucket(), 1);

    assert_eq!(
        build.table()["damage_profile"],
        json!({
            "magic_share": { "Darius": 0.0, "Filler": 0.0, "Lux": 1.0 },
            "mixed_from": 0.667,
            "magic_heavy_from": 3.0,
        })
    );
}

#[test]
fn a_magic_share_is_magic_over_total_damage_to_champions_across_games() {
    let mut bench = TableBench::new();
    bench.add_archived(1, &ArchivedMatch::new().seat(1, damage("Ezreal", 2000)));
    let mut more = ArchivedMatch::new().seat(1, damage("Ezreal", 6000));
    more = more.seat(
        2,
        Seat {
            total: 30_000,
            ..damage("Ezreal", 3000)
        },
    );
    bench.add_archived(1, &more);

    let build = build(&bench, 1);

    // 11,000 magic of 50,000.
    assert_eq!(
        build.table()["damage_profile"]["magic_share"]["Ezreal"],
        json!(0.22)
    );
}

// Mercury's Treads into a magic-heavy team and Plated Steelcaps into a physical one: the
// point of splitting by bucket.
#[test]
fn each_bucket_lists_what_was_bought_facing_it() {
    let build = build(&bench_with_one_darius_game_per_bucket(), 1);

    let list = |bucket| items(&build.candidates("Darius", "TOP", bucket).unwrap());
    assert_eq!(list("physical_heavy"), [PLATED_STEELCAPS]);
    assert_eq!(list("mixed"), [INFINITY_EDGE]);
    assert_eq!(list("magic_heavy"), [MERCURYS_TREADS]);
}

// A thin bucket's own list would be noise; the pooled one still knows the champion.
#[test]
fn a_bucket_under_the_threshold_takes_the_list_pooled_over_buckets() {
    let mut bench = bench_with_one_darius_game_per_bucket();
    bench.add_archived(1, &darius_facing(5, MERCURYS_TREADS));

    let build = build(&bench, 2);

    let pooled = vec![
        (MERCURYS_TREADS, 0.5, 9.0),
        (INFINITY_EDGE, 0.25, 9.0),
        (PLATED_STEELCAPS, 0.25, 9.0),
    ];
    assert_eq!(
        build.candidates("Darius", "TOP", "physical_heavy"),
        Some(pooled.clone())
    );
    assert_eq!(build.candidates("Darius", "TOP", "mixed"), Some(pooled));
    assert_eq!(
        build.candidates("Darius", "TOP", "magic_heavy"),
        Some(vec![(MERCURYS_TREADS, 1.0, 9.0)])
    );
}

// With too few games overall the model gets no candidates, and names no item (ADR 0004).
#[test]
fn a_champion_and_role_under_the_threshold_pooled_has_no_entry() {
    let build = build(&bench_with_one_darius_game_per_bucket(), 4);

    assert_eq!(build.candidates("Darius", "TOP", "mixed"), None);
    assert!(build.candidates(FILLER, "TOP", "mixed").is_some());
}

// Lists are by champion and role, so a game with no role must not make one.
#[test]
fn a_game_with_no_role_gives_no_list() {
    let mut bench = TableBench::new();
    let roleless = Seat { role: "", ..AHRI };
    bench.add_archived(
        2,
        &ArchivedMatch::new().seat(1, roleless).buys(1, RABADONS, 9),
    );

    let build = build(&bench, 1);

    assert_eq!(build.candidates("Ahri", "", "mixed"), None);
}

// A reworked champion's old games would list the items of a kit it no longer has.
#[test]
fn games_before_a_rework_give_no_candidates() {
    let mut bench = TableBench::new();
    bench.set_reworks(r#"[{ "champion": "Ahri", "first_valid_patch": "16.21" }]"#);
    bench.add_archived(1, &ahri_match().on_patch(16, 20).buys(1, ZHONYAS, 9));
    bench.add_archived(1, &ahri_match().buys(1, RABADONS, 9));

    let build = build(&bench, 1);

    assert_eq!(items(&only_list(&build, "Ahri", "MIDDLE")), [RABADONS]);
}

// Matches stored before payloads were archived still give their Cells.
#[test]
fn a_store_with_no_archived_match_has_cells_and_no_candidates() {
    let mut bench = TableBench::new();
    bench.add(100, &Played::new("Ahri", "GOLD"));

    let build = bench.build(&[]);

    let table = build.table();
    assert!(build.cell("Ahri", "GOLD").is_some());
    assert!(table.get("damage_profile").is_none());
    assert_eq!(table["item_candidates"], json!([]));
}

// A table without candidates would pass the publish gate and quietly end item Advice.
#[test]
fn a_build_that_cannot_reach_data_dragon_writes_no_table() {
    let mut bench = TableBench::new();
    bench.add(100, &Played::new("Ahri", "GOLD"));

    let build = bench.build_on(&FakeDataDragon::down(), &[]);

    assert!(!build.output.status.success());
    assert!(build.stderr().contains("Data Dragon"), "{}", build.stderr());
    assert!(!bench.output().exists());
}

#[test]
fn an_archive_that_does_not_read_fails_the_build_and_names_the_match() {
    let mut bench = TableBench::new();
    bench.add_archived(1, &ahri_match());
    bench.damage_archives();

    let build = build(&bench, 1);

    assert!(!build.output.status.success());
    assert!(build.stderr().contains("EUW1_1"), "{}", build.stderr());
}

// Protects the seam from Riot's real shapes, through the collector's archive, to a file
// the app still reads.
#[tokio::test]
async fn collected_matches_give_candidates_in_a_table_the_app_reads() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    let games: Vec<Game> = (0..3).map(|n| Game::new(&format!("EUW1_{n}"))).collect();
    scenario.riot.add_seed("GOLD", "seed", &games);
    scenario.run_until_all_visited(3).await;
    let reworks = Path::new(env!("CARGO_MANIFEST_DIR")).join("reworks.json");

    let args = min_games(3);
    let build = run_build(
        &scenario.database(),
        &reworks,
        scenario.dir.path(),
        &[&args[0], &args[1]],
    );

    assert_eq!(
        only_list(&build, "Champion1", "TOP"),
        [(INFINITY_EDGE, 1.0, 0.75)]
    );
    assert_eq!(
        build.table()["damage_profile"]["magic_share"]["Champion3"],
        json!(0.3)
    );
    let table = scenario.dir.path().join("table.json");
    assert!(BenchmarkTable::read(&table).is_some());
}
