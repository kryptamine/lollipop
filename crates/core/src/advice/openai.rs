use std::time::Duration;

use super::answer::Answer;
use super::responses_request::ResponsesRequest;
use super::USER_AGENT;

const CALL_TIMEOUT: Duration = Duration::from_secs(8);

/// The Responses API at a base URL, which tests point at a fake.
#[derive(Debug, Clone)]
pub(crate) struct OpenAi {
    http: reqwest::Client,
    base_url: String,
}

impl OpenAi {
    pub(super) fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// `None` for any failure: no Advice beats a wrong one (ADR 0004).
    pub(crate) async fn ask(&self, key: &str, instructions: &str, input: &str) -> Option<Answer> {
        let response = self
            .http
            .post(format!("{}/v1/responses", self.base_url))
            .bearer_auth(key)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .timeout(CALL_TIMEOUT)
            .json(&ResponsesRequest::new(instructions, input))
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        Answer::read(&response.text().await.ok()?)
    }
}
