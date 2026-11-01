/// The reason no Marker shows, for the top of the menu bar menu.
#[must_use]
pub const fn status_line(match_running: bool) -> &'static str {
    if match_running {
        "In game · No Benchmark table"
    } else {
        "No game running"
    }
}
