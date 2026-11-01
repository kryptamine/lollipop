#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Iron,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Emerald,
    Diamond,
    Master,
    Grandmaster,
    Challenger,
}

impl Tier {
    pub(super) fn parse(name: &str) -> Option<Self> {
        match name {
            "IRON" => Some(Self::Iron),
            "BRONZE" => Some(Self::Bronze),
            "SILVER" => Some(Self::Silver),
            "GOLD" => Some(Self::Gold),
            "PLATINUM" => Some(Self::Platinum),
            "EMERALD" => Some(Self::Emerald),
            "DIAMOND" => Some(Self::Diamond),
            "MASTER" => Some(Self::Master),
            "GRANDMASTER" => Some(Self::Grandmaster),
            "CHALLENGER" => Some(Self::Challenger),
            _ => None,
        }
    }

    /// How the Benchmark table names the Cell this tier falls in: Master and above are one.
    pub(crate) const fn table_label(self) -> &'static str {
        match self {
            Self::Iron => "IRON",
            Self::Bronze => "BRONZE",
            Self::Silver => "SILVER",
            Self::Gold => "GOLD",
            Self::Platinum => "PLATINUM",
            Self::Emerald => "EMERALD",
            Self::Diamond => "DIAMOND",
            Self::Master | Self::Grandmaster | Self::Challenger => "MASTER+",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Iron => "Iron",
            Self::Bronze => "Bronze",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
            Self::Platinum => "Platinum",
            Self::Emerald => "Emerald",
            Self::Diamond => "Diamond",
            Self::Master => "Master",
            Self::Grandmaster => "Grandmaster",
            Self::Challenger => "Challenger",
        }
    }
}
