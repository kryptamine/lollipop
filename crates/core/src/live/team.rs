#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Team {
    Order,
    Chaos,
}

impl Team {
    pub(super) const fn other(self) -> Self {
        match self {
            Self::Order => Self::Chaos,
            Self::Chaos => Self::Order,
        }
    }

    pub(super) fn of_player(team: &str) -> Option<Self> {
        match team {
            "ORDER" => Some(Self::Order),
            "CHAOS" => Some(Self::Chaos),
            _ => None,
        }
    }

    /// Reads a killer such as `Minion_T100L0S39N0240`, where T100 is Order and T200 Chaos.
    pub(super) fn of_minion(killer: &str) -> Option<Self> {
        if killer.starts_with("Minion_T100") {
            Some(Self::Order)
        } else if killer.starts_with("Minion_T200") {
            Some(Self::Chaos)
        } else {
            None
        }
    }

    /// The team that destroyed a structure such as `Turret_TChaos_L1_P3_...`, Chaos's own.
    pub(super) fn destroyer_of(structure: &str) -> Option<Self> {
        if structure.contains("_TChaos_") {
            Some(Self::Order)
        } else if structure.contains("_TOrder_") {
            Some(Self::Chaos)
        } else {
            None
        }
    }
}
