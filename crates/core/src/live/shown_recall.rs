use serde::Serialize;

/// A Lane opponent's trip to base, as the Enemy tracker shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShownRecall {
    pub champion: String,
    /// The time since their first purchase, as `m:ss`.
    pub ago: String,
    /// By the client's name, in the order bought.
    pub bought: Vec<String>,
}
