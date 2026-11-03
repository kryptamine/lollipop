//! Advice from OpenAI, called with the player's own key: see ADR 0016.

mod advice_call;
mod advisor;
mod answer;
mod asked;
mod failure;
mod key_check;
mod key_failure;
mod language;
mod masked_key;
mod openai;
mod question;
mod request_guard;
mod responses_request;
mod trigger;

pub use advice_call::AdviceCall;
pub use advisor::Advisor;
pub(crate) use asked::Asked;
pub use key_check::{check_key, KeyCheck};
pub use key_failure::KeyFailure;
pub use language::Language;
pub use masked_key::masked_key;
pub(crate) use question::Question;
pub use trigger::{Trigger, TRIGGERS};

pub const OPENAI_URL: &str = "https://api.openai.com";

// Fixed in code: the cost line in the Advice tab is measured on this model.
const MODEL: &str = "gpt-6-luna";
const USER_AGENT: &str = concat!("lollipop/", env!("CARGO_PKG_VERSION"));
