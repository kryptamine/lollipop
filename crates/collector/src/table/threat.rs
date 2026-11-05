use super::participant_items::ParticipantItems;

/// What an enemy team may do a lot of, which a Counter item answers.
#[derive(Clone, Copy)]
pub(super) enum Threat {
    Healing,
    Mitigation,
}

impl Threat {
    pub(super) const fn of(self, participant: &ParticipantItems) -> u64 {
        match self {
            Self::Healing => participant.heal,
            Self::Mitigation => participant.mitigated,
        }
    }
}
