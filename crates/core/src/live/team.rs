#[derive(Clone, Copy)]
pub(super) enum Team {
    Order,
    Chaos,
}

impl Team {
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
}
