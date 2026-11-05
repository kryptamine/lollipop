const SECONDS_PER_MINUTE: f64 = 60.0;

/// `gain` over the game time from `since` to `until`, a minute.
pub(super) fn per_minute(gain: f64, since: f64, until: f64) -> f64 {
    gain / ((until - since) / SECONDS_PER_MINUTE)
}
