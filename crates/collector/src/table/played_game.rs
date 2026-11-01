use lollipop_core::game_inputs::Patch;

use crate::minute_values::MinuteValues;

/// One participant's game, its frames in time order.
pub(crate) struct PlayedGame {
    pub(crate) champion: String,
    pub(crate) role: String,
    pub(crate) tier: String,
    pub(crate) patch: Patch,
    pub(crate) frames: Vec<MinuteValues>,
}
