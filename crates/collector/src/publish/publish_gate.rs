use std::collections::HashSet;

use super::table_summary::{CellKey, TableSummary};
use crate::table::Reworks;

// Provisional, see the spec. A Cell the new table gains does not make up for one it lost:
// a player on the lost one has no Marker either way.
const MAX_DROPPED_PERCENT: usize = 20;

/// Why the new table may not replace the old one, if it may not.
pub(super) fn refusal(old: &TableSummary, new: &TableSummary, reworks: &Reworks) -> Option<String> {
    // A table with `max_patch` waits for a season patch, and data past it starts over thin.
    if old
        .max_patch
        .is_some_and(|max_patch| new.newest_patch > max_patch)
    {
        return None;
    }
    // A rework excuses a loss only while the old table still holds pre-rework data: later
    // the champion's loss is like any other, however long the entry stays in the file.
    let compared: Vec<&CellKey> = old
        .cells
        .iter()
        .filter(|cell| !reworks.drops(&cell.champion, old.oldest_patch))
        .collect();
    let kept: HashSet<&CellKey> = new.cells.iter().collect();
    let dropped = compared.iter().filter(|cell| !kept.contains(*cell)).count();
    (dropped * 100 > compared.len() * MAX_DROPPED_PERCENT).then(|| {
        format!(
            "{dropped} of {} Cells would drop, over the {MAX_DROPPED_PERCENT}% gate",
            compared.len()
        )
    })
}
