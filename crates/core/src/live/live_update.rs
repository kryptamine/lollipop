use super::live_game::LiveGame;

/// What one poll saw.
#[derive(Debug)]
pub struct LiveUpdate {
    /// `None` when no match the player is part of is running.
    pub game: Option<LiveGame>,
    /// League answers nothing and holds no game: a failed poll mid-match or the
    /// next game's loading screen is not between games.
    pub between_games: bool,
}
