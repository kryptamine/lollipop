/// Numbered as `gameVersion` numbers it: patch notes call 16.21 "26.21".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    pub major: u32,
    pub minor: u32,
}

impl Patch {
    /// Reads `16.21.8342754+branch.releases-16-21...`.
    pub(super) fn parse(game_version: &str) -> Option<Self> {
        let mut numbers = game_version.split('.');
        Some(Self {
            major: numbers.next()?.parse().ok()?,
            minor: numbers.next()?.parse().ok()?,
        })
    }
}
