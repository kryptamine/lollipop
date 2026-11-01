use std::path::PathBuf;

use lollipop_core::game_inputs::Patch;

#[derive(Debug)]
pub struct PublishConfig {
    pub database: PathBuf,
    pub reworks: PathBuf,
    /// A file built earlier, published in place of a new build.
    pub table: Option<PathBuf>,
    pub max_patch: Option<Patch>,
    pub repo: String,
    pub work_dir: PathBuf,
}
