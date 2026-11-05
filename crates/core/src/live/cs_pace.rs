use serde::{Serialize, Serializer};

/// The player's CS a minute since `since`, a game time at least three minutes back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CsPace {
    pub(super) since: f64,
    pub per_minute: f64,
}

// The view shows only the value; `since` is what the Cell is compared over.
impl Serialize for CsPace {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.per_minute)
    }
}
