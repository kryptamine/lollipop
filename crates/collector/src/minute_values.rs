use crate::frame_scores::FrameScores;

/// One timeline frame of one participant. Frames run late by up to a second, so the
/// frame's own time is kept for aligning them to the minute.
pub(crate) struct MinuteValues {
    pub(crate) participant: u32,
    pub(crate) at_ms: u64,
    pub(crate) cs: u32,
    pub(crate) total_gold: u32,
    pub(crate) level: u32,
    /// `None` for a match kept before scores were.
    pub(crate) scores: Option<FrameScores>,
}
