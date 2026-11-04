use serde::Serialize;

use super::clock_text::countdown;
use super::place::Place;

// About the enemy jungler's walk from one side of the map to the other.
const CROSS_MAP: f64 = 40.0;

/// The side lane the enemy jungler cannot reach yet, from where they were just seen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeLane {
    /// `Top` or `Bot`.
    pub lane: Place,
    /// As `m:ss`.
    pub ends_in: String,
}

impl SafeLane {
    /// `None` for a sighting mid, in base or in the jungle, which is near every lane.
    pub(super) fn after(place: Place, passed: f64) -> Option<Self> {
        let lane = match place {
            Place::Bot | Place::Drake | Place::Elder => Place::Top,
            Place::Top | Place::Voidgrubs | Place::Herald | Place::Baron => Place::Bot,
            Place::Mid | Place::Jungle | Place::Base => return None,
        };
        (passed < CROSS_MAP).then(|| Self {
            lane,
            ends_in: countdown(CROSS_MAP - passed),
        })
    }
}
