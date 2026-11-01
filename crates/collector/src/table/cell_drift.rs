use super::patch_medians::PatchMedians;

// A shift past the Neutral zone changes which Marker a player would see.
const ZONE_SHARE: f64 = 0.08;
const CS_ZONE_FLOOR: f64 = 8.0;
const GOLD_ZONE_FLOOR: f64 = 150.0;

/// One Cell's medians on the two patches of a Patch window.
pub(super) struct CellDrift {
    pub(super) champion: String,
    pub(super) role: String,
    pub(super) tier: &'static str,
    pub(super) older: PatchMedians,
    pub(super) newer: PatchMedians,
}

impl CellDrift {
    pub(super) fn is_sharp(&self) -> bool {
        past_zone(self.older.cs, self.newer.cs, CS_ZONE_FLOOR)
            || past_zone(
                self.older.total_gold,
                self.newer.total_gold,
                GOLD_ZONE_FLOOR,
            )
    }

    pub(super) fn line(&self) -> String {
        format!(
            "{} {} {}: cs {:.1} to {:.1}, gold {:.0} to {:.0}",
            self.champion,
            self.role,
            self.tier,
            self.older.cs,
            self.newer.cs,
            self.older.total_gold,
            self.newer.total_gold,
        )
    }
}

fn past_zone(older: f64, newer: f64, floor: f64) -> bool {
    (newer - older).abs() > (older * ZONE_SHARE).max(floor)
}
