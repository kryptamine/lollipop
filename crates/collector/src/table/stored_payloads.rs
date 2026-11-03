use lollipop_core::game_inputs::Patch;

/// One archived match, its Match-V5 bodies still zstd JSON.
pub(crate) struct StoredPayloads {
    pub(crate) match_id: String,
    pub(crate) patch: Patch,
    pub(crate) game: Vec<u8>,
    pub(crate) timeline: Vec<u8>,
}
