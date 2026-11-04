/// An enemy in base, seen in what they bought there while alive.
pub(super) struct Recall {
    /// The game time of the first purchase.
    pub(super) at: f64,
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
