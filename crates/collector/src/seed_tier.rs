use clap::ValueEnum;

/// The eight tiers of a Cell, with Master and above merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum SeedTier {
    Iron,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Emerald,
    Diamond,
    #[value(name = "master")]
    MasterPlus,
}

impl SeedTier {
    pub(crate) const fn with_neighbours(self) -> &'static [Self] {
        match self {
            Self::Iron => &[Self::Iron, Self::Bronze],
            Self::Bronze => &[Self::Iron, Self::Bronze, Self::Silver],
            Self::Silver => &[Self::Bronze, Self::Silver, Self::Gold],
            Self::Gold => &[Self::Silver, Self::Gold, Self::Platinum],
            Self::Platinum => &[Self::Gold, Self::Platinum, Self::Emerald],
            Self::Emerald => &[Self::Platinum, Self::Emerald, Self::Diamond],
            Self::Diamond => &[Self::Emerald, Self::Diamond, Self::MasterPlus],
            Self::MasterPlus => &[Self::Diamond, Self::MasterPlus],
        }
    }

    /// How a stored match names its tier, and for all but Master+ how `league-v4` does.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Iron => "IRON",
            Self::Bronze => "BRONZE",
            Self::Silver => "SILVER",
            Self::Gold => "GOLD",
            Self::Platinum => "PLATINUM",
            Self::Emerald => "EMERALD",
            Self::Diamond => "DIAMOND",
            Self::MasterPlus => "MASTER+",
        }
    }
}
