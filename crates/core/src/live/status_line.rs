use crate::game_inputs::{ClientInputs, GameInputs, TierReading};

/// What was detected for the running game, with a reason in place of a missing value.
#[must_use]
pub fn status_line(game: Option<&GameInputs>) -> String {
    let Some(game) = game else {
        return "No game running".to_string();
    };
    let mut parts = vec![
        game.champion.as_deref().unwrap_or("Champion unknown"),
        game.role.map_or("Role unknown", |role| role.label()),
    ];
    match &game.client {
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
    parts.join(" · ")
}
