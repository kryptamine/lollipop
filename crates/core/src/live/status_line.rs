use super::live_game::LiveGame;
use super::notice::Notice;
use crate::game_inputs::{ClientInputs, TierReading};

/// What was detected for the running game, with a reason in place of a missing value.
#[must_use]
pub fn status_line(game: Option<&LiveGame>) -> String {
    let Some(game) = game else {
        return "No game running".to_string();
    };
    let inputs = &game.inputs;
    let mut parts = vec![
        inputs.champion.as_deref().unwrap_or("Champion unknown"),
        inputs.role.map_or("Role unknown", |role| role.label()),
    ];
    match &inputs.client {
        ClientInputs::Unavailable => parts.push("League client unavailable"),
        ClientInputs::Read(reading) => {
            parts.push(match reading.tier {
                TierReading::Ranked(tier) => tier.label(),
                TierReading::Unranked => "Unranked",
                TierReading::Unknown => "Tier unknown",
            });
            if reading.queue.is_none() {
                parts.push("Queue unknown");
            }
            if reading.patch.is_none() {
                parts.push("Patch unknown");
            }
        }
    }
    // The other notices repeat what the inputs above have already named.
    let beyond_inputs: Vec<String> = game
        .notices
        .iter()
        .filter(|notice| {
            matches!(
                notice,
                Notice::NoTable | Notice::StaleTable(_) | Notice::NoSchedule { .. }
            )
        })
        .map(ToString::to_string)
        .collect();
    parts.extend(beyond_inputs.iter().map(String::as_str));
    parts.join(" · ")
}
