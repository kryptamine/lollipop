use serde::Serialize;

/// What saving a pasted key came to, as the Advice tab draws it.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub(crate) enum KeySave {
    Saved {
        masked: String,
    },
    Rejected,
    Unreachable,
    /// OpenAI took the key, but the Keychain did not.
    NotKept,
}
