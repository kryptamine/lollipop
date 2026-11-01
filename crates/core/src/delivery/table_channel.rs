use std::time::Duration;

use crate::benchmarks::SCHEMA_VERSION;

pub const TABLE_CHANNEL_URL: &str = "https://pub-e9b7200aa7d442d79809d68c2fc9912e.r2.dev";

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
const USER_AGENT: &str = concat!("lollipop/", env!("CARGO_PKG_VERSION"));

/// The public bucket the collector publishes to. HTTPS alone does not vouch for what it
/// serves, which is why a manifest must carry a signature.
#[derive(Debug)]
pub struct TableChannel {
    http: reqwest::Client,
    schema_url: String,
}

impl TableChannel {
    /// # Errors
    /// If the HTTP client cannot be built, which only a broken TLS backend causes.
    pub fn new(url: &str) -> reqwest::Result<Self> {
        // Unlike the poll loop's client, this one leaves the machine: certificates are checked.
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(DOWNLOAD_TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            schema_url: format!("{url}/v{SCHEMA_VERSION}"),
        })
    }

    pub(super) async fn download(&self, object: &str) -> Option<Vec<u8>> {
        let response = self
            .http
            .get(format!("{}/{object}", self.schema_url))
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        Some(response.bytes().await.ok()?.to_vec())
    }
}
