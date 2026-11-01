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
