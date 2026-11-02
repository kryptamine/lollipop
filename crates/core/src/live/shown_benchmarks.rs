use serde::Serialize;

/// The Benchmarks the strip shows beside its Markers, `None` wherever a Marker is.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize)]
pub struct ShownBenchmarks {
    pub cs: Option<f64>,
    pub gold: Option<f64>,
    pub level: Option<f64>,
}
