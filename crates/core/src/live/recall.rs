/// An enemy's trip to base, seen in what they bought while alive.
pub(super) struct Recall {
    /// The game time of the trip's first purchase.
    pub(super) at: f64,
    /// By the client's name, in the order bought.
    pub(super) bought: Vec<String>,
}

impl Recall {
    pub(super) fn add(&mut self, bought: Vec<String>) {
        for name in bought {
            if !self.bought.contains(&name) {
                self.bought.push(name);
            }
        }
    }
}
