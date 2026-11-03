use std::time::Duration;

use reqwest::header::HeaderValue;
use reqwest::StatusCode;

use super::MODEL;

const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!("lollipop/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCheck {
    Accepted,
    Rejected,
    /// A network error, a timeout, a 429 or a 5xx: the key may well be good.
    Unreachable,
}

/// Asks OpenAI for the model Advice uses, so a key that answers 200 can call it.
pub async fn check_key(base_url: &str, key: &str) -> KeyCheck {
    // A key no header can carry would otherwise fail as a network error.
    if HeaderValue::from_str(&format!("Bearer {key}")).is_err() {
        return KeyCheck::Rejected;
    }
    let Ok(http) = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(CHECK_TIMEOUT)
        .build()
    else {
        return KeyCheck::Unreachable;
    };
    let url = format!("{}/v1/models/{MODEL}", base_url.trim_end_matches('/'));
    http.get(url)
        .bearer_auth(key)
        .send()
        .await
        .map_or(KeyCheck::Unreachable, |response| judge(response.status()))
}

// A 403 or 404 is a key that cannot use the model, so it is no better than a 401.
fn judge(status: StatusCode) -> KeyCheck {
    if status == StatusCode::OK {
        KeyCheck::Accepted
    } else if status.is_client_error() && status != StatusCode::TOO_MANY_REQUESTS {
        KeyCheck::Rejected
    } else {
        KeyCheck::Unreachable
    }
}
