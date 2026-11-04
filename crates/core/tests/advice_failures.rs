//! How Advice behaves when OpenAI fails, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use lollipop_core::advice::{AdviceCall, Advisor, KeyFailure, Language, Trigger};
use lollipop_core::live::ShownAdvice;
use serde_json::json;

use support::fake_openai::{responses_body, FakeOpenAi, Reply};
use support::hint_game::no_client;
use support::loading_game::{lineup, shown_over, LoadingGame, Updates, KEY};

const PLAN: &str = "Играй от Electrocute: размени на 3 уровне, пока у Zed нет W.";
const UNREACHABLE: &str = "Advice unavailable: could not reach OpenAI";
const NEW_KEY: &str = "sk-proj-New0123wxyz";

fn plan() -> Reply {
    Reply::advising(
        &json!({ "advice": PLAN, "next_item": null, "reason": null, "swap": null, "elixir": null })
            .to_string(),
    )
}

/// A loading screen with Advice on, and what Advice told the app.
struct Advised {
    openai: FakeOpenAi,
    game: LoadingGame,
    updates: Updates,
    advisor: Advisor,
    key: Arc<Mutex<String>>,
    log: Arc<Mutex<Vec<String>>>,
    key_failures: Arc<Mutex<Vec<KeyFailure>>>,
}

impl Advised {
    async fn replying(replies: Vec<Reply>) -> Self {
        let openai = FakeOpenAi::replying(replies).await;
        let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
        let key = Arc::new(Mutex::new(KEY.to_string()));
        let log = Arc::new(Mutex::new(Vec::new()));
        let key_failures = Arc::new(Mutex::new(Vec::new()));
        let (saved, lines, failures) = (
            Arc::clone(&key),
            Arc::clone(&log),
            Arc::clone(&key_failures),
        );
        let advisor = Advisor::new(&openai.url, move |_| {
            Some(AdviceCall {
                key: saved.lock().unwrap().clone(),
                language: Language::Russian,
            })
        })
        .logging_to(move |line| lines.lock().unwrap().push(line.to_string()))
        .on_key_failure(move |failure| failures.lock().unwrap().push(failure));
        let updates = game.poll_with(advisor.clone(), &no_client());
        Self {
            openai,
            game,
            updates,
            advisor,
            key,
            log,
            key_failures,
        }
    }

    /// Polls until OpenAI has had and answered `requests`, then a few polls more for
    /// the poll loop to take the outcome.
    async fn wait_for_requests(&mut self, requests: usize) {
        while self.openai.requests().len() < requests || self.openai.answered() < requests {
            self.updates.recv().await.unwrap();
        }
        self.settle().await;
    }

    /// Polls until the log holds `lines`: a failed ask ends with its line.
    async fn wait_for_log(&mut self, lines: usize) {
        while self.log().len() < lines {
            self.updates.recv().await.unwrap();
        }
        self.settle().await;
    }

    async fn settle(&mut self) {
        for _ in 0..3 {
            self.updates.recv().await.unwrap();
        }
    }

    /// The Widget over the first 25 s of the game.
    async fn shown_in_game(&mut self) -> Vec<Option<ShownAdvice>> {
        self.game.start_game_at(0.0);
        shown_over(&mut self.updates, 25).await
    }

    /// A game that ends and the next one's loading screen.
    async fn next_game(&mut self) {
        self.game.start_game_at(0.0);
        shown_over(&mut self.updates, 1).await;
        self.game.end_game();
        self.settle().await;
        self.game.back_to_loading_screen();
    }

    fn log(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }
}

fn says(shown: &[Option<ShownAdvice>], text: &str) -> bool {
    let shown_for = shown.iter().filter(|advice| advice.is_some()).count();
    shown_for == 20
        && shown
            .iter()
            .flatten()
            .all(|advice| advice.text == text && advice.trigger == Trigger::LoadingScreen)
}

fn shows_nothing(shown: &[Option<ShownAdvice>]) -> bool {
    shown.iter().all(Option::is_none)
}

#[tokio::test]
async fn a_5xx_is_retried_a_second_later() {
    let mut advised =
        Advised::replying(vec![Reply::status("503 Service Unavailable"), plan()]).await;
    advised.wait_for_requests(2).await;

    let gap = advised.openai.gaps()[0];
    assert!(
        gap >= Duration::from_secs(1) && gap < Duration::from_secs(2),
        "{gap:?}"
    );
    assert!(says(&advised.shown_in_game().await, PLAN));
}

#[tokio::test]
async fn a_network_error_is_retried() {
    let mut advised = Advised::replying(vec![Reply::Hangup, plan()]).await;
    advised.wait_for_requests(2).await;

    assert!(says(&advised.shown_in_game().await, PLAN));
}

// The call gives up at 8 s, twice the slowest answer measured.
#[tokio::test]
async fn a_timeout_is_retried() {
    let mut advised = Advised::replying(vec![Reply::Silence, plan()]).await;
    advised.wait_for_requests(2).await;

    let gap = advised.openai.gaps()[0];
    assert!(
        gap >= Duration::from_secs(9) && gap < Duration::from_secs(11),
        "{gap:?}"
    );
    assert!(says(&advised.shown_in_game().await, PLAN));
}

#[tokio::test]
async fn a_rate_limit_is_retried() {
    let mut advised = Advised::replying(vec![Reply::rate_limited(None), plan()]).await;
    advised.wait_for_requests(2).await;

    assert!(says(&advised.shown_in_game().await, PLAN));
}

#[tokio::test]
async fn a_short_retry_after_is_waited_out() {
    let mut advised = Advised::replying(vec![Reply::rate_limited(Some(2)), plan()]).await;
    advised.wait_for_requests(2).await;

    assert!(advised.openai.gaps()[0] >= Duration::from_secs(2));
    assert!(says(&advised.shown_in_game().await, PLAN));
}

#[tokio::test]
async fn a_long_retry_after_gets_no_retry() {
    let mut advised = Advised::replying(vec![Reply::rate_limited(Some(5)), plan()]).await;
    advised.wait_for_log(1).await;

    assert_eq!(advised.openai.requests().len(), 1);
    assert!(says(&advised.shown_in_game().await, UNREACHABLE));
}

#[tokio::test]
async fn openai_unreachable_after_the_retry_says_so() {
    let mut advised = Advised::replying(vec![Reply::status("503 Service Unavailable")]).await;
    advised.wait_for_log(1).await;

    assert_eq!(advised.openai.requests().len(), 2);
    assert!(says(&advised.shown_in_game().await, UNREACHABLE));
}

// A game that drops back to the loading screen asks again, but is still one game.
#[tokio::test]
async fn openai_unreachable_is_said_once_per_game() {
    let mut advised = Advised::replying(vec![Reply::Hangup]).await;
    advised.wait_for_log(1).await;
    assert!(says(&advised.shown_in_game().await, UNREACHABLE));

    advised.game.back_to_loading_screen();
    advised.wait_for_log(2).await;
    assert_eq!(advised.openai.requests().len(), 4);
    assert!(shows_nothing(&advised.shown_in_game().await));

    advised.next_game().await;
    advised.wait_for_log(3).await;
    assert!(says(&advised.shown_in_game().await, UNREACHABLE));
}

// A request OpenAI refused as malformed would be refused again.
#[tokio::test]
async fn a_client_error_is_not_retried_and_shows_nothing() {
    let mut advised = Advised::replying(vec![Reply::status("400 Bad Request"), plan()]).await;
    advised.wait_for_log(1).await;

    assert_eq!(advised.openai.requests().len(), 1);
    assert!(shows_nothing(&advised.shown_in_game().await));
}

#[tokio::test]
async fn a_rejected_key_stops_advice_until_a_new_key_is_saved() {
    let mut advised = Advised::replying(vec![Reply::status("401 Unauthorized"), plan()]).await;
    advised.wait_for_log(1).await;

    assert_eq!(advised.openai.requests().len(), 1);
    assert_eq!(
        *advised.key_failures.lock().unwrap(),
        [KeyFailure::Rejected]
    );
    let shown = advised.shown_in_game().await;
    assert!(says(&shown, "Advice stopped: OpenAI rejected the key"));

    advised.next_game().await;
    advised.settle().await;
    assert!(shows_nothing(&advised.shown_in_game().await));
    assert_eq!(advised.openai.requests().len(), 1);

    *advised.key.lock().unwrap() = NEW_KEY.to_string();
    advised.advisor.key_saved();
    advised.next_game().await;
    advised.wait_for_requests(2).await;
    assert!(advised.openai.requests()[1].contains("authorization: bearer sk-proj-new0123wxyz\r\n"));
    assert!(says(&advised.shown_in_game().await, PLAN));
}

// A spent quota is not a rate limit: waiting does not bring it back.
#[tokio::test]
async fn a_spent_quota_stops_advice_with_no_retry() {
    let mut advised = Advised::replying(vec![Reply::out_of_quota(), plan()]).await;
    advised.wait_for_log(1).await;

    assert_eq!(advised.openai.requests().len(), 1);
    assert_eq!(*advised.key_failures.lock().unwrap(), [KeyFailure::NoQuota]);
    let shown = advised.shown_in_game().await;
    assert!(says(
        &shown,
        "Advice stopped: the OpenAI key has no quota left"
    ));

    advised.next_game().await;
    advised.settle().await;
    assert_eq!(advised.openai.requests().len(), 1);

    // The same key, saved again once its project is paid.
    advised.advisor.key_saved();
    advised.next_game().await;
    advised.wait_for_requests(2).await;
    assert!(says(&advised.shown_in_game().await, PLAN));
}

// The runaway guard: a game that keeps returning to the loading screen.
#[tokio::test]
async fn a_game_stops_asking_after_100_requests() {
    let mut advised = Advised::replying(vec![plan()]).await;
    for asked in 1..=100 {
        advised.wait_for_requests(asked).await;
        advised.game.start_game_at(0.0);
        shown_over(&mut advised.updates, 1).await;
        advised.game.back_to_loading_screen();
    }
    advised.wait_for_log(1).await;
    for _ in 0..3 {
        advised.game.start_game_at(0.0);
        shown_over(&mut advised.updates, 1).await;
        advised.game.back_to_loading_screen();
        advised.settle().await;
    }

    assert_eq!(advised.openai.requests().len(), 100);
    assert_eq!(advised.log().len(), 1);
    assert!(advised.log()[0].contains("100 OpenAI requests"));
    assert!(shows_nothing(&advised.shown_in_game().await));

    advised.next_game().await;
    advised.wait_for_requests(101).await;
}

#[tokio::test]
async fn the_log_names_the_trigger_failure_latency_and_retry() {
    let mut advised = Advised::replying(vec![Reply::status("503 Service Unavailable")]).await;
    advised.wait_for_log(1).await;

    let line = &advised.log()[0];
    assert!(
        line.starts_with("Advice loadingScreen: http 503 after "),
        "{line}"
    );
    assert!(line.ends_with(" ms, retried: true"), "{line}");
}

// The log may end up in a bug report, so the key, prompt and answer stay out of it.
#[tokio::test]
async fn the_log_holds_no_key_prompt_or_answer() {
    let long = json!({ "advice": format!("{PLAN}{}", "а".repeat(120)), "next_item": null, "reason": null, "swap": null, "elixir": null });
    let malformed = [
        (
            Reply::with_body("200 OK", responses_body(&format!("Sure! {PLAN}"))),
            "schema",
        ),
        (Reply::advising(&long.to_string()), "length"),
    ];
    for (reply, kind) in malformed {
        let mut advised = Advised::replying(vec![reply]).await;
        advised.wait_for_log(1).await;

        assert_eq!(advised.openai.requests().len(), 1, "{kind} was retried");
        let line = &advised.log()[0];
        assert!(line.contains(&format!(": {kind} after ")), "{line}");
        for secret in [KEY, "Electrocute", "Zed", "Lane opponent"] {
            assert!(!line.contains(secret), "{secret} is in {line}");
        }
        assert!(shows_nothing(&advised.shown_in_game().await));
    }
}
