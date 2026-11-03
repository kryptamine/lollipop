#[derive(Debug)]
pub struct Widget {
    /// The key in the settings file and in the view's switches, so it must never change.
    pub id: &'static str,
}

pub const WIDGETS: [Widget; 2] = [Widget { id: "markerStrip" }, Widget { id: "timingHints" }];
