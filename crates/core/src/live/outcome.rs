use serde::{Serialize, Serializer};

/// Victory or Defeat, sent to the view as the words it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Lose,
}

impl Outcome {
    /// `Result` of `GameEnd`; anything but `Win` or `Lose` reads as no outcome.
    pub(super) fn parse(result: &str) -> Option<Self> {
        match result {
            "Win" => Some(Self::Win),
            "Lose" => Some(Self::Lose),
            _ => None,
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Win => "Victory",
            Self::Lose => "Defeat",
        }
    }
}

impl Serialize for Outcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}
