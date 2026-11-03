use std::collections::HashMap;

use super::average::{median, rounded};
use super::benchmark_file::CandidateEntry;
use super::first_purchase::FirstPurchase;
use super::item_class::ItemClass;

const COVERAGE_PERCENT: usize = 90;
const MAX_COMPLETED: usize = 12;
const MAX_BOOTS: usize = 3;
const MINUTE_MS: f64 = 60_000.0;

/// The most bought completed items, then tier-2 boots, each until they cover 90% of their
/// class's purchases or reach their cap, as one list by share.
pub(super) fn candidate_list(
    games: &[&[FirstPurchase]],
    owned_as: &HashMap<u32, Vec<u32>>,
) -> Vec<CandidateEntry> {
    let mut bought: HashMap<u32, (ItemClass, Vec<f64>)> = HashMap::new();
    for purchase in games.iter().flat_map(|purchases| purchases.iter()) {
        let (_, minutes) = bought
            .entry(purchase.item)
            .or_insert_with(|| (purchase.class, Vec::new()));
        minutes.push(millis(purchase.at_ms) / MINUTE_MS);
    }
    let mut list = [
        (ItemClass::Completed, MAX_COMPLETED),
        (ItemClass::Boots, MAX_BOOTS),
    ]
    .into_iter()
    .flat_map(|(class, cap)| covering(&bought, class, cap))
    .filter_map(|(item, minutes)| {
        Some(CandidateEntry {
            item,
            share: rounded(count(minutes.len()) / count(games.len())),
            minute: rounded(median(minutes)?),
            owned_as: owned_as.get(&item).cloned().unwrap_or_default(),
        })
    })
    .collect::<Vec<_>>();
    list.sort_by(|a, b| b.share.total_cmp(&a.share).then(a.item.cmp(&b.item)));
    list
}

fn covering(
    bought: &HashMap<u32, (ItemClass, Vec<f64>)>,
    class: ItemClass,
    cap: usize,
) -> Vec<(u32, &[f64])> {
    let mut items: Vec<(u32, &[f64])> = bought
        .iter()
        .filter(|(_, (own, _))| *own == class)
        .map(|(item, (_, minutes))| (*item, minutes.as_slice()))
        .collect();
    items.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));
    let total: usize = items.iter().map(|(_, minutes)| minutes.len()).sum();
    let mut covered = 0;
    let mut kept = Vec::new();
    for (item, minutes) in items.into_iter().take(cap) {
        if covered * 100 >= total * COVERAGE_PERCENT {
            break;
        }
        covered += minutes.len();
        kept.push((item, minutes));
    }
    kept
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "game times in milliseconds are far below 2^52"
)]
const fn millis(value: u64) -> f64 {
    value as f64
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a count of games is far below 2^52"
)]
const fn count(value: usize) -> f64 {
    value as f64
}
