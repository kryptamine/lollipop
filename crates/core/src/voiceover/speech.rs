/// What the voice must do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Speech {
    /// Read this Advice, ending a reading in progress at a word boundary.
    Read(String),
    Stop,
}
