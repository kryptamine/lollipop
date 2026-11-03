//! Data Dragon's `api/versions.json` and `item.json` documents, one struct per nested
//! object, read into the item catalog.

use std::collections::HashMap;
use std::time::Duration;

use lollipop_core::game_inputs::Patch;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::benchmark_file::ItemNameEntry;
use super::item_catalog::ItemCatalog;
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
// The support item takes three steps to reach its upgrades; the bound only stops a cycle.
const MAX_TRANSFORM_STEPS: usize = 8;

#[derive(Deserialize)]
struct ItemDocument {
    data: HashMap<String, Item>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    name: String,
    gold: Gold,
    #[serde(default)]
    from: Vec<String>,
    #[serde(default)]
    into: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    maps: HashMap<String, bool>,
    depth: Option<u32>,
    special_recipe: Option<u32>,
}

#[derive(Deserialize)]
struct Gold {
    total: u32,
    purchasable: bool,
}

#[derive(Deserialize)]
struct NameDocument {
    data: HashMap<String, Named>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
}

type Items = HashMap<u32, Item>;

/// The catalog on the newest patch only: an item that left the game, or became a
/// component, is advice nobody can follow there.
pub(super) fn item_catalog(url: &str, newest: Patch) -> Result<ItemCatalog, TableError> {
    let http = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| TableError::DataDragon(error.to_string()))?;
    let versions: Vec<String> = fetch(&http, &format!("{url}/api/versions.json"))?;
    let version = version_for(&versions, newest)?;
    let english: ItemDocument = fetch(&http, &item_json(url, version, "en_US"))?;
    let russian: NameDocument = fetch(&http, &item_json(url, version, "ru_RU"))?;
    let items: Items = english
        .data
        .into_iter()
        .filter_map(|(id, item)| Some((id.parse().ok()?, item)))
        .collect();
    let classes = items
        .iter()
        .filter_map(|(&id, item)| Some((id, class_of(item, &items)?)))
        .collect();
    let names = items
        .iter()
        .map(|(&id, item)| {
            let en = item.name.clone();
            // A name Data Dragon has not translated reads better in English than not at all.
            let ru = russian
                .data
                .get(&id.to_string())
                .map_or_else(|| en.clone(), |named| named.name.clone());
            (id, ItemNameEntry { en, ru })
        })
        .collect();
    Ok(ItemCatalog {
        version: version.to_string(),
        classes,
        owned_as: owned_as(&items),
        names,
    })
}

fn item_json(url: &str, version: &str, language: &str) -> String {
    format!("{url}/cdn/{version}/data/{language}/item.json")
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

fn on_the_rift(item: &Item) -> bool {
    item.maps.get(SUMMONERS_RIFT).copied().unwrap_or(false)
}

/// An item of the Rift no shop sells, which a purchasable one becomes in play.
fn is_transform(item: &Item) -> bool {
    on_the_rift(item) && !item.gold.purchasable
}

fn class_of(item: &Item, items: &Items) -> Option<ItemClass> {
    if !on_the_rift(item) || !item.gold.purchasable {
        return None;
    }
    let tagged = |tag: &str| item.tags.iter().any(|own| own == tag);
    if tagged("Boots") {
        return (item.depth == Some(BOOTS_TIER_2_DEPTH)).then_some(ItemClass::Boots);
    }
    // Archangel's Staff becomes Seraph's Embrace with no purchase, so it is still final.
    let builds_into_more = item.into.iter().any(|id| {
        let target = id.parse().ok().and_then(|id: u32| items.get(&id));
        !target.is_some_and(is_transform)
    });
    let completed = !builds_into_more
        && item.gold.total >= COMPLETED_MIN_GOLD
        && !NOT_COMPLETED_TAGS.iter().any(|tag| tagged(tag));
    completed.then_some(ItemClass::Completed)
}

/// Every transform, under the purchasable item its chain starts from.
fn owned_as(items: &Items) -> HashMap<u32, Vec<u32>> {
    let mut owned_as: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&id, item) in items.iter().filter(|(_, item)| is_transform(item)) {
        if let Some(source) = purchasable_source(item, items) {
            owned_as.entry(source).or_default().push(id);
        }
    }
    for transforms in owned_as.values_mut() {
        transforms.sort_unstable();
    }
    owned_as
}

/// Follows `specialRecipe`, else a lone `from`, back to an item a shop sells.
fn purchasable_source(transform: &Item, items: &Items) -> Option<u32> {
    let mut item = transform;
    for _ in 0..MAX_TRANSFORM_STEPS {
        let source = match (item.special_recipe, item.from.as_slice()) {
            (Some(source), _) => source,
            (None, [only]) => only.parse().ok()?,
            _ => return None,
        };
        item = items.get(&source)?;
        if item.gold.purchasable {
            return on_the_rift(item).then_some(source);
        }
    }
    None
}
