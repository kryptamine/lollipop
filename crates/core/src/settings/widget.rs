#[derive(Debug)]
pub struct Widget {
    /// The key in the settings file and in the view's switches, so it must never change.
    pub id: &'static str,
}

pub const ADVICE_WIDGET: &str = "adviceWidget";
pub const BUILD_WIDGET: &str = "buildWidget";

pub const WIDGETS: [Widget; 4] = [
    Widget { id: "markerStrip" },
    Widget { id: "timingHints" },
    Widget { id: BUILD_WIDGET },
    Widget { id: ADVICE_WIDGET },
];
