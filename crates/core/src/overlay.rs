//! When the overlay shows: the game is frontmost and its match is running.

pub const GAME_BUNDLE_ID: &str = "com.riotgames.LeagueofLegends.GameClient";

#[derive(Debug)]
pub struct Visibility {
    game_bundle_id: String,
    game_frontmost: bool,
    match_running: bool,
}

impl Visibility {
    #[must_use]
    pub fn new(game_bundle_id: &str) -> Self {
        Self {
            game_bundle_id: game_bundle_id.to_string(),
            game_frontmost: false,
            match_running: false,
        }
    }

    pub fn set_frontmost(&mut self, bundle_id: Option<&str>) {
        self.game_frontmost = bundle_id == Some(self.game_bundle_id.as_str());
    }

    pub const fn set_match_running(&mut self, running: bool) {
        self.match_running = running;
    }

    #[must_use]
    pub const fn shown(&self) -> bool {
        self.game_frontmost && self.match_running
    }
}
