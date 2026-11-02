/// One participant's kills, deaths and assists up to a timeline frame.
#[derive(Clone, Copy)]
pub(crate) struct FrameScores {
    pub(crate) kills: u32,
    pub(crate) deaths: u32,
    pub(crate) assists: u32,
}
