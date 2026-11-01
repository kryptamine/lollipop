use std::path::PathBuf;

#[derive(Debug)]
pub struct RollbackConfig {
    pub version: u32,
    pub bucket: String,
    pub work_dir: PathBuf,
    pub signing_key: PathBuf,
}
