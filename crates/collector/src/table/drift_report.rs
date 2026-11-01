use std::fmt;

use lollipop_core::game_inputs::Patch;

use super::cell_drift::CellDrift;
use super::drift_comparison::DriftComparison;
use super::patch_label;

/// For the owner to read: it names Cells, and never changes what the table holds.
#[derive(Debug)]
pub struct DriftReport {
    text: String,
}

impl DriftReport {
    pub(super) fn new(
        patches: &[Patch],
        comparison: Option<DriftComparison>,
        drifts: &[CellDrift],
    ) -> Self {
        let Some(comparison) = comparison else {
            let window: Vec<String> = patches.iter().copied().map(patch_label).collect();
            return Self {
                text: format!(
                    "drift report: nothing to compare, stored patches: {}",
                    window.join(", ")
                ),
            };
        };
        let mut lines = vec![format!(
            "drift report, {} to {}: Cells shifted sharply at minute 20: {}",
            patch_label(comparison.older),
            patch_label(comparison.newer),
            drifts.len()
        )];
        lines.extend(drifts.iter().map(CellDrift::line));
        Self {
            text: lines.join("\n"),
        }
    }
}

impl fmt::Display for DriftReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}
