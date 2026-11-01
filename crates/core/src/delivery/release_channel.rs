use std::time::Duration;

use serde::Deserialize;

use super::release_token::ReleaseToken;
use crate::benchmarks::SCHEMA_VERSION;

pub const GITHUB_API: &str = "https://api.github.com";
pub const RELEASE_REPO: &str = "kryptamine/lollipop";

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
// GitHub refuses a request with no user agent.
const USER_AGENT: &str = concat!("lollipop/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Asset {
    id: u64,
    name: String,
}

/// The Release of this schema: its tag is the `v1` of the `v1/manifest.json` path.
#[derive(Debug)]
pub struct ReleaseChannel {
    http: reqwest::Client,
    releases: String,
    token: ReleaseToken,
}

impl ReleaseChannel {
    /// # Errors
    /// If the HTTP client cannot be built, which only a broken TLS backend causes.
    pub fn new(api: &str, repo: &str, token: ReleaseToken) -> reqwest::Result<Self> {
        // Unlike the poll loop's client, this one leaves the machine: certificates are checked.
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(DOWNLOAD_TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            releases: format!("{api}/repos/{repo}/releases"),
            token,
        })
    }

    pub(super) async fn assets(&self) -> Option<Vec<Asset>> {
        let url = format!("{}/tags/v{SCHEMA_VERSION}", self.releases);
        let body = self.get(&url, "application/vnd.github+json").await?;
        let release: Release = serde_json::from_slice(&body).ok()?;
        Some(release.assets)
    }

    /// GitHub redirects to storage that refuses a token, and the client drops the token on
    /// a redirect to another host.
    pub(super) async fn download(&self, assets: &[Asset], name: &str) -> Option<Vec<u8>> {
        let asset = assets.iter().find(|asset| asset.name == name)?;
        let url = format!("{}/assets/{}", self.releases, asset.id);
        self.get(&url, "application/octet-stream").await
    }

    async fn get(&self, url: &str, accept: &str) -> Option<Vec<u8>> {
        let response = self
            .http
            .get(url)
            .bearer_auth(self.token.secret())
            .header(reqwest::header::ACCEPT, accept)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        Some(response.bytes().await.ok()?.to_vec())
    }
}
