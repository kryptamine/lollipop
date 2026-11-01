#[derive(Debug)]
pub struct Widget {
    /// The key in the settings file and in the view's switches, so it must never change.
    pub id: &'static str,
    pub label: &'static str,
}

pub const WIDGETS: [Widget; 1] = [Widget {
    id: "markerStrip",
    label: "Marker strip",
}];
