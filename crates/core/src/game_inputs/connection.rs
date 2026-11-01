use std::time::Duration;

use serde::de::DeserializeOwned;

use super::lockfile::Lockfile;

// A timeout counts as unavailable for the whole game, and a game loading can slow
// the client down, so this is well above the Live Client Data poll's own timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct Connection<'a> {
    http: &'a reqwest::Client,
    lockfile: Lockfile,
}

impl<'a> Connection<'a> {
    pub(super) const fn new(http: &'a reqwest::Client, lockfile: Lockfile) -> Self {
        Self { http, lockfile }
    }

    /// `None` for no answer, an error status or a body that does not read as `T`.
    pub(super) async fn get<T: DeserializeOwned>(&self, path: &str) -> Option<T> {
        let Lockfile {
            port,
            password,
            protocol,
        } = &self.lockfile;
        self.http
            .get(format!("{protocol}://127.0.0.1:{port}{path}"))
            .basic_auth("riot", Some(password))
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .json()
            .await
            .ok()
    }
}
