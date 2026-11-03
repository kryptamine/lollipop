//! The player's OpenAI key checked against a fake OpenAI before it is stored.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::advice::{check_key, masked_key, KeyCheck};
use tokio::net::TcpListener;

use support::fake_openai::FakeOpenAi;

const KEY: &str = "sk-proj-Fake0123abcd";

#[tokio::test]
async fn a_key_that_reads_the_advice_model_is_accepted() {
    let openai = FakeOpenAi::answering("200 OK").await;

    assert_eq!(check_key(&openai.url, KEY).await, KeyCheck::Accepted);
    let requests = openai.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("get /v1/models/gpt-6-luna "));
    assert!(requests[0].contains("authorization: bearer sk-proj-fake0123abcd\r\n"));
}

// `make dev-mock` hands over a base URL with a trailing slash.
#[tokio::test]
async fn a_base_url_with_a_trailing_slash_reaches_the_same_path() {
    let openai = FakeOpenAi::answering("200 OK").await;

    assert_eq!(
        check_key(&format!("{}/", openai.url), KEY).await,
        KeyCheck::Accepted
    );
    assert!(openai.requests()[0].starts_with("get /v1/models/gpt-6-luna "));
}

#[tokio::test]
async fn a_401_rejects_the_key() {
    let openai = FakeOpenAi::answering("401 Unauthorized").await;

    assert_eq!(check_key(&openai.url, KEY).await, KeyCheck::Rejected);
}

// A key whose project cannot use the model would fail every Advice call.
#[tokio::test]
async fn a_key_without_the_model_is_rejected() {
    let openai = FakeOpenAi::answering("404 Not Found").await;

    assert_eq!(check_key(&openai.url, KEY).await, KeyCheck::Rejected);
}

#[tokio::test]
async fn a_5xx_means_openai_could_not_be_reached() {
    let openai = FakeOpenAi::answering("503 Service Unavailable").await;

    assert_eq!(check_key(&openai.url, KEY).await, KeyCheck::Unreachable);
}

// A rate limit says nothing about the key, so it must not read as a rejection.
#[tokio::test]
async fn a_429_means_openai_could_not_be_reached() {
    let openai = FakeOpenAi::answering("429 Too Many Requests").await;

    assert_eq!(check_key(&openai.url, KEY).await, KeyCheck::Unreachable);
}

#[tokio::test]
async fn a_network_error_means_openai_could_not_be_reached() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);

    assert_eq!(check_key(&url, KEY).await, KeyCheck::Unreachable);
}

// A pasted line break cannot go into a header, and is not worth a request.
#[tokio::test]
async fn a_key_no_header_can_carry_is_rejected_without_a_request() {
    let openai = FakeOpenAi::answering("200 OK").await;

    assert_eq!(
        check_key(&openai.url, "sk-one\nsk-two").await,
        KeyCheck::Rejected
    );
    assert!(openai.requests().is_empty());
}

#[test]
fn a_masked_key_shows_only_its_last_four_characters() {
    assert_eq!(masked_key(KEY), "sk-…abcd");
    assert_eq!(masked_key("abc"), "sk-…abc");
}
