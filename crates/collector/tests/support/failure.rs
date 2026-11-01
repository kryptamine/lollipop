#[derive(Clone, Copy)]
pub struct Failure {
    pub status: u16,
    pub retry_after_s: Option<u64>,
}
