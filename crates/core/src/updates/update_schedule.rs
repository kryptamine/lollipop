use super::UpdateStep;

/// Checks at launch and after each game, and installs a downloaded update only
/// between games, since installing relaunches the app.
#[derive(Debug)]
pub struct UpdateSchedule {
    between_games: bool,
    check_due: bool,
    downloaded: bool,
}

impl UpdateSchedule {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            between_games: false,
            check_due: true,
            downloaded: false,
        }
    }

    pub const fn set_between_games(&mut self, between_games: bool) -> Option<UpdateStep> {
        self.between_games = between_games;
        if !between_games {
            self.check_due = true;
            return None;
        }
        if self.downloaded {
            self.downloaded = false;
            return Some(UpdateStep::Install);
        }
        if !self.check_due {
            return None;
        }
        self.check_due = false;
        Some(UpdateStep::Check)
    }

    pub const fn update_downloaded(&mut self) -> Option<UpdateStep> {
        if !self.between_games {
            self.downloaded = true;
            return None;
        }
        Some(UpdateStep::Install)
    }
}

impl Default for UpdateSchedule {
    fn default() -> Self {
        Self::new()
    }
}
