use serde_json::{json, Value};

pub const ENOUGH_GAMES: u32 = 300;

/// A Benchmark table file as `lollipop-collector build-table` writes it.
pub fn table_of(cells: &[Value]) -> String {
    json!({ "schema_version": 1, "patch_window": ["16.20", "16.21"], "cells": cells }).to_string()
}

pub fn cell(champion: &str, role: &str, tier: &str, minutes: &[Value]) -> Value {
    json!({ "champion": champion, "role": role, "tier": tier, "minutes": minutes })
}

pub fn minute(minute: u32, cs: f64, gold: f64, level: f64, n: u32) -> Value {
    json!({ "minute": minute, "cs": cs, "total_gold": gold, "level": level, "n": n })
}

/// The same Benchmarks at every minute of a 40 minute game.
pub fn flat_minutes(cs: f64, gold: f64, level: f64) -> Vec<Value> {
    (0..=40)
        .map(|at| minute(at, cs, gold, level, ENOUGH_GAMES))
        .collect()
}
