/// Time left as `m:ss`, rounded up so the last second shown is `0:01`.
pub(super) fn countdown(left: f64) -> String {
    clock(left.ceil())
}

/// Time passed as `m:ss`, rounded down.
pub(super) fn ago(passed: f64) -> String {
    clock(passed.floor())
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "no std conversion from f64; a whole number of seconds, never negative"
)]
fn clock(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
