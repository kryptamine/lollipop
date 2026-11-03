use serde::Serialize;

/// A moment in a game that asks for Advice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    LoadingScreen,
    Death,
}

pub const TRIGGERS: [Trigger; 2] = [Trigger::LoadingScreen, Trigger::Death];

impl Trigger {
    /// The key in the settings file and in the view's switches, so it must never change.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::LoadingScreen => "loadingScreen",
            Self::Death => "death",
        }
    }

    /// An answer replaces the Advice showing when its rank is the same or higher.
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::LoadingScreen => 0,
            Self::Death => 1,
        }
    }
}
