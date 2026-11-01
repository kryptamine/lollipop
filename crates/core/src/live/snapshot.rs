use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Snapshot {
    pub cs: u32,
}
