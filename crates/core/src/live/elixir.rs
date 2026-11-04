/// A 500-gold consumable worth buying once the inventory is full, before Baron or Elder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Elixir {
    Iron,
    Sorcery,
    Wrath,
}

pub(super) const ELIXIR_COST: f64 = 500.0;

impl Elixir {
    pub(super) const ALL: [Self; 3] = [Self::Iron, Self::Sorcery, Self::Wrath];

    pub(super) fn of_item(item: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|elixir| elixir.item() == item)
    }

    /// Code picks by the champion's damage alone: Iron is only ever the model's pick.
    pub(super) fn by_magic_share(share: f64) -> Self {
        if share >= 0.5 {
            Self::Sorcery
        } else {
            Self::Wrath
        }
    }

    pub(super) const fn item(self) -> u32 {
        match self {
            Self::Iron => 2138,
            Self::Sorcery => 2139,
            Self::Wrath => 2140,
        }
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Iron => "Elixir of Iron",
            Self::Sorcery => "Elixir of Sorcery",
            Self::Wrath => "Elixir of Wrath",
        }
    }
}
