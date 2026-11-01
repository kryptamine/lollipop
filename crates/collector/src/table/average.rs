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
