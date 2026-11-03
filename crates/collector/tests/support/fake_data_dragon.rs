use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use serde_json::{json, Map, Value};

pub const INFINITY_EDGE: u32 = 3031;
pub const RABADONS: u32 = 3089;
pub const ZHONYAS: u32 = 3157;
pub const MERCURYS_TREADS: u32 = 3111;
pub const PLATED_STEELCAPS: u32 = 3047;
pub const BOOTS: u32 = 1001;
/// The tier-3 boots Mercury's Treads build into.
pub const TIER_3_BOOTS: u32 = 3172;
pub const DORANS_BLADE: u32 = 1055;
pub const ARCHANGELS_STAFF: u32 = 3003;
/// Archangel's Staff with full mana, named by `specialRecipe`.
pub const SERAPHS_EMBRACE: u32 = 3040;
pub const MANAMUNE: u32 = 3004;
/// Manamune with full mana, named by its lone `from`.
pub const MURAMANA: u32 = 3042;
/// A completed item that transforms twice, as the support item does.
pub const QUEST_ITEM: u32 = 6800;
pub const QUEST_ITEM_STEP_1: u32 = 6801;
pub const QUEST_ITEM_STEP_2: u32 = 6802;
/// Over 2200 gold, and still a component of Infinity Edge.
pub const COSTLY_COMPONENT: u32 = 6632;
/// Builds into the costly component, so Infinity Edge holds it two levels down.
pub const LONG_SWORD: u32 = 1036;
/// Builds straight into Infinity Edge.
pub const BF_SWORD: u32 = 1038;
pub const HEALTH_POTION: u32 = 2003;
pub const ARENA_ITEM: u32 = 443_054;
/// Completed on 16.20 only, as an item that left the game with 16.21.
pub const REMOVED_ITEM: u32 = 6630;
/// Completed on 16.20, and a component on 16.21.
pub const DEMOTED_ITEM: u32 = 6631;
/// Sixteen more completed items from here, and five more tier-2 boots from `MORE_BOOTS`.
pub const MORE_COMPLETED: u32 = 6600;
pub const MORE_BOOTS: u32 = 6700;

/// Stands in for Data Dragon on a thread of its own, so a blocking test can use it too.
/// It lists no 16.22, so a build on 16.22 reads 16.21.1.
pub struct FakeDataDragon {
    pub url: String,
}

impl FakeDataDragon {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let _ = answer(stream.unwrap());
            }
        });
        Self { url }
    }

    /// A port nothing listens on.
    pub fn down() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        Self { url }
    }
}

fn answer(stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut line = String::new();
    while reader.read_line(&mut line)? > 2 {
        line.clear();
    }
    let path = request_line.split(' ').nth(1).unwrap_or_default();
    let body = if path == "/api/versions.json" {
        Some(json!(["16.21.1", "16.20.1", "16.19.1", "16.1.1"]))
    } else {
        item_document(path)
    };
    let mut stream = stream;
    let Some(body) = body else {
        return stream.write_all(b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n");
    };
    let body = body.to_string();
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// The `en_US` document in full, and the `ru_RU` one as just names, less Zhonya's.
fn item_document(path: &str) -> Option<Value> {
    let rest = path.strip_prefix("/cdn/")?.strip_suffix("/item.json")?;
    let (version, language) = rest.split_once("/data/")?;
    let mut document = items(version == "16.20.1");
    match language {
        "en_US" => Some(document),
        "ru_RU" => {
            let data = document["data"].as_object_mut()?;
            data.remove(&ZHONYAS.to_string());
            for (id, item) in data.iter_mut() {
                *item = json!({ "name": format!("Предмет {id}") });
            }
            Some(document)
        }
        _ => None,
    }
}

fn items(older_patch: bool) -> Value {
    let mut data = Map::new();
    let mut add = |id: u32, mut item: Value| {
        item["name"] = json!(format!("Item {id}"));
        data.insert(id.to_string(), item);
    };
    add(
        INFINITY_EDGE,
        built_from(
            item(3450, &["CriticalStrike"], &[], None),
            &["6632", "1038"],
        ),
    );
    add(RABADONS, item(3600, &["SpellDamage"], &[], None));
    add(ZHONYAS, item(3250, &["SpellDamage", "Armor"], &[], None));
    add_boots(&mut add);
    add(DORANS_BLADE, item(450, &["Lane"], &[], None));
    add(
        COSTLY_COMPONENT,
        built_from(item(2300, &["Damage"], &["3031"], Some(2)), &["1036"]),
    );
    add(LONG_SWORD, item(350, &["Damage"], &["6632"], Some(1)));
    add(BF_SWORD, item(1300, &["Damage"], &["3031"], Some(1)));
    add(
        ARCHANGELS_STAFF,
        item(2900, &["SpellDamage"], &["3040"], Some(3)),
    );
    add(
        SERAPHS_EMBRACE,
        transform(&["SpellDamage"], Some(ARCHANGELS_STAFF), &["3003"]),
    );
    add(MANAMUNE, item(2900, &["Damage"], &["3042"], Some(3)));
    add(MURAMANA, transform(&["Damage"], None, &["3004"]));
    add(QUEST_ITEM, item(2500, &["GoldPer"], &["6801"], None));
    add(
        QUEST_ITEM_STEP_1,
        transform(&["GoldPer"], Some(QUEST_ITEM), &[]),
    );
    add(QUEST_ITEM_STEP_2, transform(&["GoldPer"], None, &["6801"]));
    add(HEALTH_POTION, item(50, &["Consumable"], &[], None));
    let mut arena = item(3000, &["Damage"], &[], None);
    arena["maps"] = json!({ "11": false, "30": true });
    add(ARENA_ITEM, arena);
    for index in 0..16 {
        add(MORE_COMPLETED + index, item(3000, &["Damage"], &[], None));
    }
    if older_patch {
        add(REMOVED_ITEM, item(3000, &["Damage"], &[], None));
        add(DEMOTED_ITEM, item(3000, &["Damage"], &[], None));
    } else {
        add(DEMOTED_ITEM, item(1300, &["Damage"], &["3031"], Some(2)));
    }
    json!({ "type": "item", "version": "16.21.1", "data": data })
}

fn add_boots(add: &mut impl FnMut(u32, Value)) {
    add(
        MERCURYS_TREADS,
        built_from(item(1250, &["Boots"], &["3172"], Some(2)), &["1001"]),
    );
    add(
        TIER_3_BOOTS,
        built_from(item(1750, &["Boots"], &[], Some(3)), &["3111"]),
    );
    add(PLATED_STEELCAPS, item(1200, &["Boots"], &["3174"], Some(2)));
    add(BOOTS, item(300, &["Boots"], &["3111", "3047"], None));
    for index in 0..5 {
        add(MORE_BOOTS + index, item(1100, &["Boots"], &[], Some(2)));
    }
}

fn item(total: u32, tags: &[&str], into: &[&str], depth: Option<u32>) -> Value {
    let mut item = json!({
        "gold": { "base": 100, "purchasable": true, "total": total, "sell": total / 2 },
        "tags": tags,
        "maps": { "11": true, "12": true },
    });
    if !into.is_empty() {
        item["into"] = json!(into);
    }
    if let Some(depth) = depth {
        item["depth"] = json!(depth);
    }
    item
}

fn built_from(mut item: Value, from: &[&str]) -> Value {
    item["from"] = json!(from);
    item
}

/// An item no shop sells, which `special_recipe`, else its lone `from`, becomes.
fn transform(tags: &[&str], special_recipe: Option<u32>, from: &[&str]) -> Value {
    let mut item = json!({
        "gold": { "base": 0, "purchasable": false, "total": 3000, "sell": 2100 },
        "tags": tags,
        "maps": { "11": true, "12": true },
        "inStore": false,
    });
    if let Some(source) = special_recipe {
        item["specialRecipe"] = json!(source);
    }
    if !from.is_empty() {
        item["from"] = json!(from);
    }
    item
}
