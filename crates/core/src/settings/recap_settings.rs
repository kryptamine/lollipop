use serde::{Deserialize, Serialize};

/// The Recap's switches, both on until the player turns them off.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecapSettings {
    /// Whether the Recap window opens on its own after a game.
    pub window: bool,
    /// Whether a Recap asks OpenAI for its Lessons.
    pub lessons: bool,
}

impl Default for RecapSettings {
    fn default() -> Self {
        Self {
            window: true,
            lessons: true,
        }
    }
}
