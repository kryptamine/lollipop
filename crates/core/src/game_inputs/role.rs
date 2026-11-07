use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

impl Role {
    /// Live Client Data writes `JUNGLE` where the League client writes `jungle`.
    /// `None` covers the empty and `NONE` values of a game with no assigned role.
    pub(crate) fn parse(position: &str) -> Option<Self> {
        match position.to_ascii_uppercase().as_str() {
            "TOP" => Some(Self::Top),
            "JUNGLE" => Some(Self::Jungle),
            "MIDDLE" => Some(Self::Mid),
            "BOTTOM" => Some(Self::Bottom),
            "UTILITY" => Some(Self::Support),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Jungle => "Jungle",
            Self::Mid => "Mid",
            Self::Bottom => "Bot",
            Self::Support => "Support",
        }
    }
}

/// As the view shows it, by its label.
impl Serialize for Role {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}
