use super::compared_game::MARKER_QUEUES;
use super::game_hints::HINT_QUEUES;
use super::notice::Notice;
use super::objective_schedule::ObjectiveSchedule;
use crate::benchmarks::BenchmarkTable;
use crate::game_inputs::{ClientInputs, ClientReading, GameInputs, Patch, TierReading};

/// A queue with no Markers or hints, or an unknown one, loses nothing worth a notice.
/// A thin Cell gets none either. An unavailable client hides the rest, as it has no queue.
pub(super) fn notices_of(inputs: &GameInputs, table: Option<&BenchmarkTable>) -> Vec<Notice> {
    let ClientInputs::Read(client) = &inputs.client else {
        return vec![Notice::ClientUnavailable];
    };
    let Some(queue) = client.queue else {
        return Vec::new();
    };
    let mut notices = Vec::new();
    if MARKER_QUEUES.contains(&queue) {
        marker_notices(inputs, client, table, &mut notices);
    } else if HINT_QUEUES.contains(&queue) && client.patch.is_none() {
        notices.push(Notice::PatchUnknown);
    }
    if HINT_QUEUES.contains(&queue) {
        notices.extend(client.patch.and_then(schedule_notice));
    }
    notices
}

fn marker_notices(
    inputs: &GameInputs,
    client: &ClientReading,
    table: Option<&BenchmarkTable>,
    notices: &mut Vec<Notice>,
) {
    match (client.patch, table) {
        (None, _) => notices.push(Notice::PatchUnknown),
        (Some(_), None) => notices.push(Notice::NoTable),
        (Some(patch), Some(table)) if !table.serves(patch) => {
            notices.push(Notice::StaleTable(patch));
        }
        (Some(_), Some(_)) => {}
    }
    if inputs.role.is_none() {
        notices.push(Notice::RoleUnknown);
    }
    match client.tier {
        TierReading::Ranked(_) => {}
        TierReading::Unranked => notices.push(Notice::Unranked),
        TierReading::Unknown => notices.push(Notice::TierUnknown),
    }
}

fn schedule_notice(patch: Patch) -> Option<Notice> {
    ObjectiveSchedule::for_patch(patch)
        .is_none()
        .then_some(Notice::NoSchedule {
            season: patch.major,
        })
}
