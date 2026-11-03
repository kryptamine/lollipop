use std::path::PathBuf;

use lollipop_core::game_inputs::Patch;

pub const DATA_DRAGON_URL: &str = "https://ddragon.leagueoflegends.com";
// About ±0.1 on a share; a smaller bucket takes its champion and role's pooled list.
pub const MIN_CANDIDATE_GAMES: usize = 50;

#[derive(Debug)]
pub struct TableConfig {
    pub database: PathBuf,
    pub reworks: PathBuf,
    pub output: PathBuf,
    pub max_patch: Option<Patch>,
    pub data_dragon_url: String,
    /// The fewest games an Item candidate list is built from.
    pub min_candidate_games: usize,
}
