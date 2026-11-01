use std::path::PathBuf;

#[derive(Debug)]
pub struct RollbackConfig {
    pub version: u32,
    pub repo: String,
    pub work_dir: PathBuf,
}
