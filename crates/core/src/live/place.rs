use serde::Serialize;

/// Where the enemy jungler was last seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Place {
    Top,
    Mid,
    Bot,
    Jungle,
    Base,
    Drake,
    Elder,
    Herald,
    Baron,
}

impl Place {
    /// The lane of a champion playing `position`, where a kill of them most likely was.
    pub(super) fn of_position(position: &str) -> Option<Self> {
        match position {
            "TOP" => Some(Self::Top),
            "MIDDLE" => Some(Self::Mid),
            "BOTTOM" | "UTILITY" => Some(Self::Bot),
            "JUNGLE" => Some(Self::Jungle),
            _ => None,
        }
    }
}
