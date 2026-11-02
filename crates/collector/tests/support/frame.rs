pub struct Frame {
    pub at_ms: u64,
    pub cs: u32,
    pub total_gold: u32,
    pub level: u32,
    /// Kills, deaths and assists, `None` for a game kept before scores were.
    pub scores: Option<[u32; 3]>,
}
