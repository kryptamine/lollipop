use serde::Serialize;

use super::marker::Marker;

/// `None` is no Marker at all: nothing to compare, unlike a neutral one.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Markers {
    pub cs: Option<Marker>,
    pub gold: Option<Marker>,
    pub level: Option<Marker>,
}
