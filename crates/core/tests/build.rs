//! The standing Build, with League faked over HTTP and no OpenAI key.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::BuildItem;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedReceiver;

use support::item_game::{
    item_table, magic_heavy, physical_heavy, table_with_cryptbloom, with_threats, ItemGame,
    CRYPTBLOOM,
};
use support::loading_game::ME;

const SORCERERS: (u32, &str) = (3020, "Сапоги чародея");
const BERSERKERS: (u32, &str) = (3006, "Берсерки");
const STEELCAPS: (u32, &str) = (3047, "Бронированные сапоги");
const BANSHEES: (u32, &str) = (3102, "Покров банши");
const ZHONYAS: (u32, &str) = (3157, "Песочные часы Жони");
const SERAPHS: (u32, &str) = (3040, "Объятия серафима");
const RABADONS: (u32, &str) = (3089, "Смертельная шляпа Рабадона");
const VOID_STAFF: (u32, &str) = (3135, "Посох пустоты");
const MORELLO: (u32, &str) = (3165, "Морелономикон");
const ROD: (u32, &str) = (1058, "Огромный жезл");
const TOME: (u32, &str) = (1052, "Усиливающий фолиант");
const INFINITY_EDGE: (u32, &str) = (3031, "Грань бесконечности");
const DOMINIKS: (u32, &str) = (3036, "Последний довод лорда Доминика");
const THORNMAIL: (u32, &str) = (3075, "Шипованный доспех");
const LUX: &str = "Foe4#EUW";

const FULL_BUILD: [u32; 6] = [3020, 3102, 3157, 3003, 3089, 3135];

type Builds = UnboundedReceiver<Vec<BuildItem>>;

/// The Build once the poll loop has seen the game as it stands.
async fn build_now(builds: &mut Builds) -> Vec<u32> {
    let mut last = Vec::new();
    for _ in 0..3 {
        last = builds.recv().await.unwrap();
    }
    last.iter().map(|shown| shown.item).collect()
}

/// A game at 5:00 on `table`, polled until its Build stands.
async fn running(table: &Value) -> (ItemGame, Builds) {
    let game = ItemGame::running_at(300.0, table).await;
    let mut builds = game.poll_builds();
    build_now(&mut builds).await;
    (game, builds)
}

// The player plans the build before the game starts, and needs no OpenAI key for it.
#[tokio::test]
async fn the_build_shows_from_the_loading_screen() {
    let game = ItemGame::at_loading_screen(&item_table(&magic_heavy())).await;
    let mut builds = game.poll_builds();

    let shown = builds.recv().await.unwrap();
    assert_eq!(shown.len(), 6);
    assert_eq!(
        shown[0],
        BuildItem {
            item: 3020,
            icon: Some("https://ddragon.leagueoflegends.com/cdn/16.21.1/img/item/3020.png".into()),
            name: Some("Сапоги чародея".into()),
        }
    );
    assert_eq!(build_now(&mut builds).await, FULL_BUILD);

    game.start_game();
    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

// Boots by share and five completed items by share, then all of them by median minute:
// Steelcaps come earlier but are bought less, and the Rod is no completed item.
#[tokio::test]
async fn the_build_takes_the_largest_shares_in_order_of_minute() {
    let (_game, mut builds) = running(&item_table(&magic_heavy())).await;

    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

#[tokio::test]
async fn the_build_follows_the_enemy_damage_profile() {
    let (_game, mut builds) = running(&item_table(&physical_heavy())).await;

    assert_eq!(build_now(&mut builds).await, [3047, 3157]);
}

// Lux usually deals magic damage, but two items of attack damage halve her share
// twice over, and the team sinks into the mixed bucket.
#[tokio::test]
async fn the_build_follows_the_items_enemies_build() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.give_items(LUX, &[INFINITY_EDGE, DOMINIKS]);
    assert_eq!(build_now(&mut builds).await, [3020, 3157]);
}

#[tokio::test]
async fn an_enemy_tank_item_leaves_the_profile_alone() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.give_items(LUX, &[THORNMAIL]);
    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

// A Tome bought for Banshee's would be wasted if the new bucket dropped it.
#[tokio::test]
async fn a_started_next_item_outlasts_a_new_profile() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;
    game.buy(&[SORCERERS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089, 3135]);
    game.buy(&[TOME], 0.0);
    build_now(&mut builds).await;

    game.give_items(LUX, &[INFINITY_EDGE, DOMINIKS]);
    assert_eq!(build_now(&mut builds).await, [3102, 3157]);
}

// Seraph's Embrace is the Archangel's Staff it grew from.
#[tokio::test]
async fn a_held_item_drops_from_the_build() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.buy(&[ZHONYAS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3020, 3102, 3003, 3089, 3135]);
    game.buy(&[SERAPHS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3020, 3102, 3089, 3135]);
}

// A component is no finished item, so buying one changes nothing.
#[tokio::test]
async fn a_component_leaves_the_build_whole() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.buy(&[ROD], 0.0);
    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

#[tokio::test]
async fn an_off_build_item_drops_the_next_item() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;
    game.buy(&[SORCERERS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089, 3135]);

    game.buy(&[MORELLO], 0.0);
    assert_eq!(build_now(&mut builds).await, [3157, 3003, 3089, 3135]);
}

// Bought or sold later, the item an off-build one replaced stays out, and takes no
// other item's place.
#[tokio::test]
async fn a_dropped_item_never_returns() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;
    game.buy(&[SORCERERS, MORELLO], 0.0);
    assert_eq!(build_now(&mut builds).await, [3157, 3003, 3089, 3135]);

    game.give_items(ME, &[SORCERERS]);
    assert_eq!(build_now(&mut builds).await, [3157, 3003, 3089, 3135]);
    game.buy(&[BANSHEES], 0.0);
    assert_eq!(build_now(&mut builds).await, [3157, 3003, 3089, 3135]);
}

#[tokio::test]
async fn an_item_finished_early_leaves_the_next_item_leading() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.buy(&[RABADONS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3020, 3102, 3157, 3003, 3135]);
}

#[tokio::test]
async fn off_build_boots_drop_the_builds_boots() {
    let (game, mut builds) = running(&item_table(&magic_heavy())).await;

    game.buy(&[STEELCAPS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089, 3135]);
    game.give_items(ME, &[]);
    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089, 3135]);
}

// An app started mid-game finds items already held: they take free slots, but drop
// nothing, and a player with boots needs no second pair.
#[tokio::test]
async fn the_build_never_shows_more_items_than_free_slots() {
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    game.give_items(ME, &[BERSERKERS, MORELLO]);
    let mut builds = game.poll_builds();

    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089]);
}

#[tokio::test]
async fn a_full_inventory_hides_the_build() {
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    game.give_items(ME, &[BERSERKERS, MORELLO, BANSHEES, ZHONYAS, RABADONS]);
    let mut builds = game.poll_builds();
    assert_eq!(build_now(&mut builds).await, [3003]);

    game.buy(&[VOID_STAFF], 0.0);
    assert!(build_now(&mut builds).await.is_empty());
}

// No Benchmark table, or a Stale one, has no Item candidates to build from: see ADR 0004.
#[tokio::test]
async fn a_missing_or_stale_table_gives_no_build() {
    let (_game, mut builds) = running(&json!({})).await;
    assert!(build_now(&mut builds).await.is_empty());

    let mut stale = item_table(&magic_heavy());
    stale["patch_window"] = json!(["16.10"]);
    let (_game, mut builds) = running(&stale).await;
    assert!(build_now(&mut builds).await.is_empty());
}

fn ahri_dealing_magic() -> Value {
    let mut shares = magic_heavy();
    shares["Ahri"] = json!(0.9);
    shares
}

// Void Staff has the larger share, but the games that build Zhonya's and Banshee's go
// to Morello instead.
#[tokio::test]
async fn the_build_takes_the_items_games_buy_together() {
    let mut table = item_table(&magic_heavy());
    table["item_candidates"][0]["usual_builds"] = json!({
        "physical_heavy": [],
        "mixed": [],
        "magic_heavy": [3157, 3102, 3165, 3003, 3089],
    });
    let (_game, mut builds) = running(&table).await;

    assert_eq!(
        build_now(&mut builds).await,
        [3020, 3102, 3165, 3157, 3003, 3089]
    );
}

// Grievous Wounds answers a team that heals a lot; the last item, bought least surely,
// makes room for it.
#[tokio::test]
async fn heavy_healing_puts_anti_heal_in_place_of_the_last_item() {
    let table = with_threats(item_table(&magic_heavy()), true, false);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(
        build_now(&mut builds).await,
        [3020, 3102, 3165, 3157, 3003, 3089]
    );
}

#[tokio::test]
async fn light_healing_and_mitigation_leave_the_build_alone() {
    let table = with_threats(table_with_cryptbloom(&ahri_dealing_magic()), false, false);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

// Ahri deals magic damage, so armor penetration would do her nothing.
#[tokio::test]
async fn heavy_mitigation_puts_penetration_of_the_players_damage_in_the_build() {
    let table = with_threats(table_with_cryptbloom(&ahri_dealing_magic()), false, true);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(
        build_now(&mut builds).await,
        [3020, 3102, 3157, 3003, 3089, CRYPTBLOOM]
    );
}

// With no magic share the kind of penetration is a guess, and a wrong one is worse.
#[tokio::test]
async fn a_player_with_no_magic_share_gets_no_penetration() {
    let table = with_threats(table_with_cryptbloom(&magic_heavy()), false, true);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

// A Counter item put in is never the item the next one takes the place of, even when
// it comes last: Morello is bought late here.
#[tokio::test]
async fn each_counter_item_takes_the_place_of_another_item() {
    let mut table = with_threats(table_with_cryptbloom(&ahri_dealing_magic()), true, true);
    let list = table["item_candidates"][0]["magic_heavy"]
        .as_array_mut()
        .unwrap();
    let morello = list
        .iter_mut()
        .find(|item| item["item"] == MORELLO.0)
        .unwrap();
    morello["minute"] = json!(30.0);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(
        build_now(&mut builds).await,
        [3020, 3102, 3157, 3003, CRYPTBLOOM, 3165]
    );
}

#[tokio::test]
async fn a_build_that_counters_already_takes_no_counter_item() {
    let mut table = with_threats(item_table(&magic_heavy()), true, false);
    table["counters"]["anti_heal"] = json!([RABADONS.0, MORELLO.0]);
    let (_game, mut builds) = running(&table).await;

    assert_eq!(build_now(&mut builds).await, FULL_BUILD);
}

// Morello bought early is still the anti-heal the team calls for.
#[tokio::test]
async fn a_player_holding_a_counter_item_takes_no_other() {
    let table = with_threats(item_table(&magic_heavy()), true, false);
    let game = ItemGame::running_at(300.0, &table).await;
    game.give_items(ME, &[MORELLO]);
    let mut builds = game.poll_builds();

    assert_eq!(build_now(&mut builds).await, [3020, 3102, 3157, 3003, 3089]);
}

const CHEMPUNK: (u32, &str) = (6609, "Химтековый гнилокол");
const EXECUTIONERS: (u32, &str) = (3123, "Призыв палача");

/// Heavy healing, with Chempunk as a second anti-heal item Ahri rarely buys, built from
/// the Executioner's Calling.
fn table_with_chempunk() -> Value {
    let mut table = with_threats(item_table(&magic_heavy()), true, false);
    let list = table["item_candidates"][0]["magic_heavy"]
        .as_array_mut()
        .unwrap();
    list.push(json!({ "item": CHEMPUNK.0, "share": 0.05, "minute": 19.0, "builds_from": [3123] }));
    table["completed"]
        .as_array_mut()
        .unwrap()
        .push(json!(CHEMPUNK.0));
    table["counters"]["anti_heal"] = json!([MORELLO.0, CHEMPUNK.0]);
    table
}

// Executioner's Calling already bought would be wasted on another anti-heal item.
#[tokio::test]
async fn a_held_component_picks_the_counter_item_it_builds_into() {
    let game = ItemGame::running_at(300.0, &table_with_chempunk()).await;
    game.give_items(ME, &[EXECUTIONERS]);
    let mut builds = game.poll_builds();

    assert_eq!(
        build_now(&mut builds).await,
        [3020, 3102, CHEMPUNK.0, 3157, 3003, 3089]
    );
}

// The player answered the healing with another anti-heal item, so Morello has no job.
#[tokio::test]
async fn an_off_build_counter_item_takes_the_counter_items_place() {
    let (game, mut builds) = running(&table_with_chempunk()).await;
    game.buy(&[SORCERERS], 0.0);
    build_now(&mut builds).await;

    game.buy(&[CHEMPUNK], 0.0);
    assert_eq!(build_now(&mut builds).await, [3102, 3157, 3003, 3089]);
}

// Dominik's takes Zhonya's place, not Morello's, which the team still calls for.
#[tokio::test]
async fn an_off_build_item_never_drops_a_counter_item() {
    let table = with_threats(item_table(&magic_heavy()), true, false);
    let (game, mut builds) = running(&table).await;
    game.buy(&[SORCERERS, BANSHEES], 0.0);
    assert_eq!(build_now(&mut builds).await, [3165, 3157, 3003, 3089]);

    game.buy(&[DOMINIKS], 0.0);
    assert_eq!(build_now(&mut builds).await, [3165, 3003, 3089]);
}

// A new bucket makes the Build again from its usual build, and asks for Counter items
// again: Morello is no usual buy against a mixed team.
#[tokio::test]
async fn a_new_bucket_builds_from_its_usual_build_with_counter_items() {
    let mut table = with_threats(item_table(&magic_heavy()), true, false);
    let mixed = table["item_candidates"][0]["mixed"].as_array_mut().unwrap();
    mixed.push(json!({ "item": MORELLO.0, "share": 0.1, "minute": 18.0 }));
    table["item_candidates"][0]["usual_builds"] = json!({
        "physical_heavy": [],
        "mixed": [3157],
        "magic_heavy": [],
    });
    let (game, mut builds) = running(&table).await;

    game.give_items(LUX, &[INFINITY_EDGE, DOMINIKS]);
    assert_eq!(build_now(&mut builds).await, [3020, 3165, 3157]);
}
