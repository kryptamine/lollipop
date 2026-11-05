// The API floors CS to 10, so the true value is on average 5 above it.
const CS_OFFSET: f64 = 5.0;

/// The likeliest true CS behind the API's floored value.
pub(super) fn likely_cs(cs: u32) -> f64 {
    f64::from(cs) + CS_OFFSET
}
