/// `None` with no values. An even count takes the mean of the two in the middle.
pub(super) fn median(values: &[f64]) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let upper = sorted.len() / 2;
    let high = *sorted.get(upper)?;
    if sorted.len() % 2 == 1 {
        return Some(high);
    }
    let low = *sorted.get(upper - 1)?;
    Some(f64::midpoint(low, high))
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a count of games is far below 2^52"
)]
pub(super) fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

// Two decimals is finer than any Neutral zone, and keeps noise digits out of the file.
pub(super) fn rounded(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Linear between the two nearest of sorted `values`, `None` with none.
pub(super) fn quantile(values: &[f64], share: f64) -> Option<f64> {
    let last = values.len().checked_sub(1)?;
    let position = share * count(last);
    let below = position.floor();
    let low = *values.get(index(below))?;
    let high = values.get(index(below) + 1).copied().unwrap_or(low);
    Some((high - low).mul_add(position - below, low))
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a count of values is far below 2^52"
)]
const fn count(value: usize) -> f64 {
    value as f64
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a whole, non-negative position within the slice"
)]
const fn index(position: f64) -> usize {
    position as usize
}
