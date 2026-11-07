use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::advice_call::AdviceCall;
use super::answer::Answer;
use super::asked::Asked;
use super::failure::Failure;
use super::key_failure::KeyFailure;
use super::language::Language;
use super::lessons_answer::read_lessons;
use super::lessons_off::LessonsOff;
use super::openai::OpenAi;
use super::question::Question;
use super::request_guard::{Request, RequestGuard, REQUESTS_PER_GAME};
use super::responses_request::ResponsesRequest;
use super::trigger::Trigger;

// Twice the measured worst answer of 3.7 s.
const CALL_TIMEOUT: Duration = Duration::from_secs(8);
const RETRY_WAIT: Duration = Duration::from_secs(1);
// A retry with less than this before the deadline would rarely answer in time.
const RETRY_LEFT: Duration = Duration::from_secs(4);
const UNREACHABLE: &str = "Advice unavailable: could not reach OpenAI";

type CallFor = Arc<dyn Fn(Trigger) -> Option<AdviceCall> + Send + Sync>;
type LessonsCall = Arc<dyn Fn() -> Result<AdviceCall, LessonsOff> + Send + Sync>;
type Log = Arc<dyn Fn(&str) + Send + Sync>;
type OnKeyFailure = Arc<dyn Fn(KeyFailure) + Send + Sync>;
type LanguageNow = Arc<dyn Fn() -> Language + Send + Sync>;

/// Where Advice comes from. `call_for` is asked as each Trigger fires, so a switch
/// flipped or a key saved mid-game applies to the next one.
#[derive(Clone)]
pub struct Advisor {
    openai: OpenAi,
    call_for: CallFor,
    lessons_call: LessonsCall,
    log: Log,
    on_key_failure: OnKeyFailure,
    language: LanguageNow,
    guard: Arc<Mutex<RequestGuard>>,
}

impl Advisor {
    pub fn new(
        base_url: &str,
        call_for: impl Fn(Trigger) -> Option<AdviceCall> + Send + Sync + 'static,
    ) -> Self {
        Self {
            openai: OpenAi::new(base_url),
            call_for: Arc::new(call_for),
            lessons_call: Arc::new(|| Err(LessonsOff::NoKey)),
            log: Arc::new(|_| {}),
            on_key_failure: Arc::new(|_| {}),
            language: Arc::new(|| Language::English),
            guard: Arc::default(),
        }
    }

    /// Each line names the Trigger or the Lessons, never the key, the prompt or the answer.
    #[must_use]
    pub fn logging_to(mut self, log: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.log = Arc::new(log);
        self
    }

    /// Asked as each Recap is made, as `call_for` is for a Trigger. With none, no
    /// Lessons are asked for.
    #[must_use]
    pub fn reviewing(
        mut self,
        lessons_call: impl Fn() -> Result<AdviceCall, LessonsOff> + Send + Sync + 'static,
    ) -> Self {
        self.lessons_call = Arc::new(lessons_call);
        self
    }

    #[must_use]
    pub fn on_key_failure(mut self, react: impl Fn(KeyFailure) + Send + Sync + 'static) -> Self {
        self.on_key_failure = Arc::new(react);
        self
    }

    /// Names the Build Widget shows follow it, with no key needed.
    #[must_use]
    pub fn in_language(mut self, language: impl Fn() -> Language + Send + Sync + 'static) -> Self {
        self.language = Arc::new(language);
        self
    }

    pub(crate) fn language(&self) -> Language {
        (self.language)()
    }

    /// `None` also once OpenAI refused the key, until [`Self::key_saved`].
    pub(crate) fn call_for(&self, trigger: Trigger) -> Option<AdviceCall> {
        if self.guard().key_failed() {
            return None;
        }
        (self.call_for)(trigger)
    }

    pub(crate) fn lessons_call(&self) -> Result<AdviceCall, LessonsOff> {
        if self.guard().key_failed() {
            return Err(LessonsOff::KeyRefused);
        }
        (self.lessons_call)()
    }

    /// Even the key OpenAI refused: a spent quota comes back once the project is paid.
    pub fn key_saved(&self) {
        self.guard().set_key_failed(false);
    }

    pub(crate) fn end_game(&self) {
        self.guard().end_game();
    }

    pub(crate) fn log(&self, line: &str) {
        (self.log)(line);
    }

    pub(crate) async fn ask(&self, call: &AdviceCall, question: &Question<'_>) -> Asked {
        let request = ResponsesRequest::new(question.instructions, question.input);
        match self.answer(call, question, &request, Answer::read).await {
            Some(Ok(answer)) => Asked::Answered(answer),
            Some(Err(failure)) => self.after(failure),
            None => Asked::Nothing,
        }
    }

    /// A Recap's Lessons, `None` when OpenAI failed, as the log says. A failure here
    /// never spends the next game's one "could not reach OpenAI".
    pub(crate) async fn review(
        &self,
        call: &AdviceCall,
        question: &Question<'_>,
    ) -> Option<Vec<String>> {
        let request = ResponsesRequest::for_lessons(question.instructions, question.input);
        match self.answer(call, question, &request, read_lessons).await? {
            Ok(lessons) => Some(lessons),
            Err(failure) => {
                if let Some(key_failure) = failure.key_failure() {
                    self.refuse_key(key_failure);
                }
                None
            }
        }
    }

    /// One retry for a failure it may mend, if the deadline leaves time for it. `None`
    /// once the game's requests are spent.
    async fn answer<T>(
        &self,
        call: &AdviceCall,
        question: &Question<'_>,
        request: &ResponsesRequest<'_>,
        read: fn(&str) -> Result<T, Failure>,
    ) -> Option<Result<T, Failure>> {
        let started = Instant::now();
        let mut retried = false;
        loop {
            let taken = self.guard().take_request();
            match taken {
                Request::Granted => {}
                Request::CapReached => {
                    (self.log)(&format!(
                        "Advice {}: dropped, {REQUESTS_PER_GAME} OpenAI requests this game",
                        question.asker
                    ));
                    return None;
                }
                Request::Refused => return None,
            }
            let timeout =
                CALL_TIMEOUT.min(question.deadline.saturating_duration_since(Instant::now()));
            let failure = match self.attempt(call, request, timeout, read).await {
                Ok(answer) => return Some(Ok(answer)),
                Err(failure) => failure,
            };
            if let Some(wait) = retry_wait(failure, question.deadline).filter(|_| !retried) {
                tokio::time::sleep(wait).await;
                retried = true;
                continue;
            }
            (self.log)(&format!(
                "Advice {}: {failure} after {} ms, retried: {retried}",
                question.asker,
                started.elapsed().as_millis()
            ));
            return Some(Err(failure));
        }
    }

    async fn attempt<T>(
        &self,
        call: &AdviceCall,
        request: &ResponsesRequest<'_>,
        timeout: Duration,
        read: fn(&str) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        if timeout.is_zero() {
            return Err(Failure::Timeout);
        }
        self.openai.attempt(&call.key, request, timeout, read).await
    }

    fn after(&self, failure: Failure) -> Asked {
        if let Some(key_failure) = failure.key_failure() {
            self.refuse_key(key_failure);
            return Asked::Unavailable(key_failure.widget_text());
        }
        if failure.is_unreachable() && self.guard().say_unreachable() {
            return Asked::Unavailable(UNREACHABLE);
        }
        Asked::Nothing
    }

    fn refuse_key(&self, failure: KeyFailure) {
        self.guard().set_key_failed(true);
        (self.on_key_failure)(failure);
    }

    fn guard(&self) -> MutexGuard<'_, RequestGuard> {
        self.guard.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A `Retry-After` over [`RETRY_LEFT`] gets no retry, nor does a retry that would
/// leave less than that before the deadline.
fn retry_wait(failure: Failure, deadline: Instant) -> Option<Duration> {
    let wait = match failure {
        Failure::RateLimited {
            retry_after: Some(retry_after),
        } if retry_after > RETRY_LEFT => return None,
        Failure::RateLimited {
            retry_after: Some(retry_after),
        } => retry_after.max(RETRY_WAIT),
        _ if failure.is_unreachable() => RETRY_WAIT,
        _ => return None,
    };
    let left = deadline.saturating_duration_since(Instant::now() + wait);
    (left >= RETRY_LEFT).then_some(wait)
}

impl fmt::Debug for Advisor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Advisor")
            .field("openai", &self.openai)
            .finish_non_exhaustive()
    }
}
