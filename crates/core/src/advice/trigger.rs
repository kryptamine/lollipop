use serde::Serialize;

/// A moment in a game that asks for Advice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    LoadingScreen,
}

pub const TRIGGERS: [Trigger; 1] = [Trigger::LoadingScreen];

impl Trigger {
    /// The key in the settings file and in the view's switches, so it must never change.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::LoadingScreen => "loadingScreen",
        }
    }
}
