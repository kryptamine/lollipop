use std::collections::HashMap;
use std::path::Path;

use lollipop_core::game_inputs::Patch;
use serde::Deserialize;

use super::table_error::TableError;

#[derive(Deserialize)]
struct Entry {
    champion: String,
    first_valid_patch: String,
}

/// The first valid patch per reworked champion, from the file in the repository.
pub(super) struct Reworks {
    first_valid: HashMap<String, Patch>,
}

impl Reworks {
    /// A file that does not read fails the build: going on would serve a reworked champion
    /// its old Benchmark.
    pub(super) fn read(path: &Path) -> Result<Self, TableError> {
        let refuse = |reason: String| TableError::Reworks(path.into(), reason);
        let body = std::fs::read_to_string(path).map_err(|error| refuse(error.to_string()))?;
        let entries: Vec<Entry> =
            serde_json::from_str(&body).map_err(|error| refuse(error.to_string()))?;
        let mut first_valid = HashMap::new();
        for entry in entries {
            let first = Patch::parse(&entry.first_valid_patch)
                .ok_or_else(|| refuse(format!("bad patch {}", entry.first_valid_patch)))?;
            if first_valid.insert(entry.champion.clone(), first).is_some() {
                return Err(refuse(format!("{} is listed twice", entry.champion)));
            }
        }
        Ok(Self { first_valid })
    }

    pub(super) fn drops(&self, champion: &str, patch: Patch) -> bool {
        self.first_valid
            .get(champion)
            .is_some_and(|first_valid| patch < *first_valid)
    }
}
