//! Data Dragon's `api/versions.json` and `item.json` documents, one struct per nested object.

use std::collections::HashMap;
use std::time::Duration;

use lollipop_core::game_inputs::Patch;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::item_class::ItemClass;
use super::patch_label;
use super::table_error::TableError;

const USER_AGENT: &str = concat!("lollipop-collector/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const SUMMONERS_RIFT: &str = "11";
// Research heuristic: it keeps every completed item on 16.21 but Mejai's.
const COMPLETED_MIN_GOLD: u32 = 2200;
const BOOTS_TIER_2_DEPTH: u32 = 2;
const NOT_COMPLETED_TAGS: [&str; 3] = ["Boots", "Consumable", "Trinket"];

#[derive(Deserialize)]
struct ItemDocument {
    data: HashMap<String, Item>,
}

#[derive(Deserialize)]
struct Item {
    gold: Gold,
    #[serde(default)]
    into: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    maps: HashMap<String, bool>,
    depth: Option<u32>,
}

#[derive(Deserialize)]
struct Gold {
    total: u32,
    purchasable: bool,
}

/// The class of every item a candidate list may hold, on the newest patch only: an item
/// that left the game, or became a component, is advice nobody can follow there.
pub(super) fn item_classes(
    url: &str,
    newest: Patch,
) -> Result<HashMap<u32, ItemClass>, TableError> {
    let http = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| TableError::DataDragon(error.to_string()))?;
    let versions: Vec<String> = fetch(&http, &format!("{url}/api/versions.json"))?;
    let version = version_for(&versions, newest)?;
    let path = format!("{url}/cdn/{version}/data/en_US/item.json");
    let document: ItemDocument = fetch(&http, &path)?;
    let classes = document.data.iter().filter_map(|(id, item)| {
        let id = id.parse().ok()?;
        Some((id, class_of(item)?))
    });
    Ok(classes.collect())
}

fn fetch<T: DeserializeOwned>(http: &Client, url: &str) -> Result<T, TableError> {
    let refuse = |error: reqwest::Error| TableError::DataDragon(error.without_url().to_string());
    let response = http.get(url).send().map_err(refuse)?;
    response
        .error_for_status()
        .map_err(refuse)?
        .json()
        .map_err(refuse)
}

/// Newest first, as Data Dragon lists them. A patch Data Dragon has not caught up with
/// takes the newest version before it.
fn version_for(versions: &[String], patch: Patch) -> Result<&str, TableError> {
    versions
        .iter()
        .find(|version| Patch::parse(version).is_some_and(|listed| listed <= patch))
        .map(String::as_str)
        .ok_or_else(|| TableError::DataDragon(format!("no version up to {}", patch_label(patch))))
}

fn class_of(item: &Item) -> Option<ItemClass> {
    let on_the_rift = item.maps.get(SUMMONERS_RIFT).copied().unwrap_or(false);
    if !on_the_rift || !item.gold.purchasable {
        return None;
    }
    let tagged = |tag: &str| item.tags.iter().any(|own| own == tag);
    if tagged("Boots") {
        return (item.depth == Some(BOOTS_TIER_2_DEPTH)).then_some(ItemClass::Boots);
    }
    let completed = item.into.is_empty()
        && item.gold.total >= COMPLETED_MIN_GOLD
        && !NOT_COMPLETED_TAGS.iter().any(|tag| tagged(tag));
    completed.then_some(ItemClass::Completed)
}
