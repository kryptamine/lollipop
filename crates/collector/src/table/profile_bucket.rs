/// The enemy team's Damage profile bucket.
#[derive(Clone, Copy)]
pub(super) enum ProfileBucket {
    PhysicalHeavy,
    Mixed,
    MagicHeavy,
}

impl ProfileBucket {
    pub(super) const ALL: [Self; 3] = [Self::PhysicalHeavy, Self::Mixed, Self::MagicHeavy];

    pub(super) const fn index(self) -> usize {
        match self {
            Self::PhysicalHeavy => 0,
            Self::Mixed => 1,
            Self::MagicHeavy => 2,
        }
    }
}
