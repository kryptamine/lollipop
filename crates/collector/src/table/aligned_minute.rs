use crate::minute_values::MinuteValues;

const MINUTE_MS: u64 = 60_000;

/// One participant's values at an exact minute of game time.
pub(super) struct AlignedMinute {
    pub(super) minute: usize,
    pub(super) cs: f64,
    pub(super) total_gold: f64,
    pub(super) level: f64,
    /// Kills, deaths and assists, when both frames around the minute have them.
    pub(super) scores: Option<[f64; 3]>,
}

/// Interpolates to each whole minute the frames span. No minute follows the last frame,
/// the game's end, so a game counts only towards the minutes it reached.
pub(super) fn align_to_minutes(frames: &[MinuteValues]) -> Vec<AlignedMinute> {
    let Some(last) = frames.last() else {
        return Vec::new();
    };
    let mut aligned = Vec::new();
    let mut minute: usize = 0;
    let mut at_ms: u64 = 0;
    while at_ms <= last.at_ms {
        aligned.extend(values_at(frames, minute, at_ms));
        minute += 1;
        at_ms += MINUTE_MS;
    }
    aligned
}

fn values_at(frames: &[MinuteValues], minute: usize, at_ms: u64) -> Option<AlignedMinute> {
    let next = frames.partition_point(|frame| frame.at_ms <= at_ms);
    let before = frames.get(next.checked_sub(1)?)?;
    let Some(after) = frames.get(next) else {
        return Some(between(before, before, minute, 0.0));
    };
    let share = millis(at_ms - before.at_ms) / millis(after.at_ms - before.at_ms);
    Some(between(before, after, minute, share))
}

fn between(
    before: &MinuteValues,
    after: &MinuteValues,
    minute: usize,
    share: f64,
) -> AlignedMinute {
    let blend = |from: u32, to: u32| {
        let (from, to) = (f64::from(from), f64::from(to));
        (to - from).mul_add(share, from)
    };
    let scores = before.scores.zip(after.scores).map(|(from, to)| {
        [
            blend(from.kills, to.kills),
            blend(from.deaths, to.deaths),
            blend(from.assists, to.assists),
        ]
    });
    AlignedMinute {
        minute,
        cs: blend(before.cs, after.cs),
        total_gold: blend(before.total_gold, after.total_gold),
        level: blend(before.level, after.level),
        scores,
    }
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "game times in milliseconds are far below 2^52"
)]
const fn millis(value: u64) -> f64 {
    value as f64
}
