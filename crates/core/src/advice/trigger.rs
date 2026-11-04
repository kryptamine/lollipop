use serde::Serialize;

/// A moment in a game that asks for Advice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    LoadingScreen,
    TurningPoint,
    Death,
}

pub const TRIGGERS: [Trigger; 3] = [
    Trigger::LoadingScreen,
    Trigger::TurningPoint,
    Trigger::Death,
];

impl Trigger {
    /// The key in the settings file and in the view's switches, so it must never change.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::LoadingScreen => "loadingScreen",
            Self::TurningPoint => "turningPoint",
            Self::Death => "death",
        }
    }

    /// An answer replaces the Advice showing when its rank is the same or higher.
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::LoadingScreen => 0,
            Self::TurningPoint => 1,
            Self::Death => 2,
        }
    }
}
