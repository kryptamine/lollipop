#[derive(Clone)]
pub struct Request {
    pub path: String,
    pub at_ms: u64,
    pub token: Option<String>,
    pub user_agent: Option<String>,
}
