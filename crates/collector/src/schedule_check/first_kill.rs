use lollipop_core::game_inputs::Patch;

/// The earliest kill of one objective across a patch's stored matches.
pub(crate) struct FirstKill {
    pub(crate) patch: Patch,
    pub(crate) objective: String,
    pub(crate) at_ms: u32,
}
