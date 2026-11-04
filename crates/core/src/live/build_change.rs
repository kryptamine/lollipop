const SHOWN_FOR: f64 = 10.0;

/// Why a model replaced the Next item, shown under the Build Widget for a few seconds.
pub(super) struct BuildChange {
    pub(super) reason: String,
    /// `None` until the game starts: the loading screen hides the overlay.
    from: Option<f64>,
}

impl BuildChange {
    pub(super) const fn new(reason: String, game_time: Option<f64>) -> Self {
        Self {
            reason,
            from: game_time,
        }
    }

    /// One that came before `GameStart` counts its time from `GameStart`.
    pub(super) fn showing_at(&mut self, game_time: f64, game_start: Option<f64>) -> bool {
        let from = *self
            .from
            .get_or_insert_with(|| game_start.unwrap_or(game_time));
        game_time < from + SHOWN_FOR
    }
}
