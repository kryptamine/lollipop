use std::path::PathBuf;

use lollipop_core::game_inputs::Patch;

#[derive(Debug)]
pub struct TableConfig {
    pub database: PathBuf,
    pub reworks: PathBuf,
    pub output: PathBuf,
    pub max_patch: Option<Patch>,
}
