use super::patch::Patch;
use super::tier_reading::TierReading;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientReading {
    pub queue: Option<u32>,
    pub patch: Option<Patch>,
    pub tier: TierReading,
}
