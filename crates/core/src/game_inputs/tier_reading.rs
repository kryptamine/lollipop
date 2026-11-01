use super::tier::Tier;

/// `Unknown` means the client gave no usable answer; `Unranked` is an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierReading {
    Ranked(Tier),
    Unranked,
    Unknown,
}
