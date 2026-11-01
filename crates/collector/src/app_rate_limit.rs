// Held below Riot's count, so skew against Riot's own window does not earn a 429.
const HELD_PERCENT: u64 = 90;

/// One window of `x-app-rate-limit`.
#[derive(Clone, Copy)]
pub(crate) struct AppRateLimit {
    pub(crate) max_calls: u64,
    pub(crate) window_ms: u64,
}

impl AppRateLimit {
    /// Reads `100:120,20:1`, a count and seconds per window. `None` if any is malformed.
    pub(crate) fn parse_all(header: &str) -> Option<Vec<Self>> {
        header.split(',').map(Self::parse).collect()
    }

    fn parse(window: &str) -> Option<Self> {
        let (count, seconds) = window.split_once(':')?;
        let count: u64 = count.parse().ok()?;
        let seconds: u64 = seconds.parse().ok()?;
        Some(Self {
            max_calls: (count * HELD_PERCENT / 100).max(1),
            window_ms: seconds * 1000,
        })
    }
}
