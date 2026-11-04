use serde::Serialize;

/// A Lane opponent's Recall, as the Enemy tracker shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownRecall {
    pub champion: String,
    /// An estimate of their walk back to lane, as `m:ss`.
    pub back_in: String,
    /// They hold Teleport, so they may be back sooner.
    pub teleport: bool,
    /// By the client's name, in the order bought.
    pub bought: Vec<String>,
}
