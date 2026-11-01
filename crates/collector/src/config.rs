use std::fmt;
use std::path::PathBuf;

use crate::SeedTier;

pub struct Config {
    pub tier: SeedTier,
    pub key: String,
    pub database: PathBuf,
    /// Serves `league-v4`, such as `https://euw1.api.riotgames.com`.
    pub platform_url: String,
    /// Serves `match-v5`, such as `https://europe.api.riotgames.com`.
    pub regional_url: String,
}

// By hand, so the key never reaches a log.
impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("tier", &self.tier)
            .finish_non_exhaustive()
    }
}
