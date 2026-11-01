use std::fmt;

use crate::game_inputs::Patch;

/// Why a game lacks Markers or hints. Each displays as its own line in the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Notice {
    ClientUnavailable,
    PatchUnknown,
    NoTable,
    StaleTable(Patch),
    RoleUnknown,
    TierUnknown,
    Unranked,
    NoSchedule { season: u32 },
}

impl fmt::Display for Notice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientUnavailable => f.write_str("League client unavailable"),
            Self::PatchUnknown => f.write_str("Patch unknown"),
            Self::NoTable => f.write_str("No Benchmarks yet"),
            Self::StaleTable(Patch { major, minor }) => {
                write!(f, "Benchmarks out of date (patch {major}.{minor})")
            }
            Self::RoleUnknown => f.write_str("Role unknown"),
            Self::TierUnknown => f.write_str("Tier unknown"),
            Self::Unranked => f.write_str("Unranked"),
            Self::NoSchedule { season } => write!(f, "No schedule for Season {season}"),
        }
    }
}
