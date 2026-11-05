/// The player's CS a minute since `since`, a game time at least three minutes back.
#[derive(Debug, Clone, Copy)]
pub(super) struct RecentCs {
    pub(super) since: f64,
    pub(super) per_minute: f64,
}
