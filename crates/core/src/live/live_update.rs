use super::build_item::BuildItem;
use super::live_game::LiveGame;

/// What one poll saw.
#[derive(Debug)]
pub struct LiveUpdate {
    /// `None` when no match the player is part of is running.
    pub game: Option<LiveGame>,
    /// From the loading screen on, as the Build Widget shows it; empty with no Build.
    pub build: Vec<BuildItem>,
    /// Why a model just replaced the Next item, for the few seconds it shows.
    pub build_change: Option<String>,
    /// League answers nothing and holds no game: a failed poll mid-match or the
    /// next game's loading screen is not between games.
    pub between_games: bool,
}
