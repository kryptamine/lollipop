use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub cs: u32,
}
