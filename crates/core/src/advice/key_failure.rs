use serde::Serialize;

/// Why OpenAI refused the saved key in a game. Advice stops until a key is saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyFailure {
    Rejected,
    NoQuota,
}

impl KeyFailure {
    pub(super) const fn widget_text(self) -> &'static str {
        match self {
            Self::Rejected => "Advice stopped: OpenAI rejected the key",
            Self::NoQuota => "Advice stopped: the OpenAI key has no quota left",
        }
    }
}
