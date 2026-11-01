use std::path::Path;

use lollipop_core::benchmarks::BenchmarkTable;
use lollipop_core::game_inputs::Patch;
use serde::Deserialize;

#[derive(Deserialize)]
struct TableFile {
    max_patch: Option<String>,
    patch_window: Vec<String>,
    cells: Vec<CellKey>,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub(super) struct CellKey {
    pub(super) champion: String,
    role: String,
    tier: String,
}

/// What the publish gate compares of a table file.
pub(super) struct TableSummary {
    pub(super) max_patch: Option<Patch>,
    pub(super) oldest_patch: Patch,
    pub(super) newest_patch: Patch,
    pub(super) cells: Vec<CellKey>,
}

impl TableSummary {
    /// Reads through the app's own reader first: a file it would reject must not ship.
    pub(super) fn read(path: &Path) -> Result<Self, String> {
        let name = path.display();
        BenchmarkTable::read(path).ok_or_else(|| format!("{name} does not read"))?;
        let body = std::fs::read(path).map_err(|error| format!("{name}: {error}"))?;
        let file: TableFile =
            serde_json::from_slice(&body).map_err(|error| format!("{name}: {error}"))?;
        let parse = |label: &String| {
            Patch::parse(label).ok_or_else(|| format!("{name}: bad patch {label}"))
        };
        let window = file
            .patch_window
            .iter()
            .map(parse)
            .collect::<Result<Vec<_>, _>>()?;
        let (Some(oldest_patch), Some(newest_patch)) =
            (window.iter().min().copied(), window.iter().max().copied())
        else {
            return Err(format!("{name} has an empty patch window"));
        };
        Ok(Self {
            max_patch: file.max_patch.as_ref().map(parse).transpose()?,
            oldest_patch,
            newest_patch,
            cells: file.cells,
        })
    }
}
