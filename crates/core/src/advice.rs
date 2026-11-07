//! Advice from OpenAI, called with the player's own key: see ADR 0016.

mod advice_call;
mod advisor;
mod answer;
mod answer_parts;
mod asked;
mod failure;
mod key_check;
mod key_failure;
mod language;
mod lessons_answer;
mod lessons_off;
mod masked_key;
mod openai;
mod proposed_change;
mod proposed_swap;
mod question;
mod request_guard;
mod responses_reply;
mod responses_request;
mod trigger;

pub use advice_call::AdviceCall;
pub use advisor::Advisor;
pub(crate) use answer_parts::AnswerParts;
pub use key_check::{check_key, KeyCheck};
pub use key_failure::KeyFailure;
pub use language::Language;
pub use lessons_off::LessonsOff;
pub use masked_key::masked_key;
pub(crate) use proposed_change::ProposedChange;
pub(crate) use proposed_swap::ProposedSwap;
pub(crate) use question::Question;
pub use trigger::{Trigger, TRIGGERS};

/// What every Trigger tells the model before its own part.
pub(crate) const COACHING: &str =
    "You coach a League of Legends player in their game. Answer with one concrete \
idea the player can act on, at most 120 characters, written in the language the input \
names, as `advice`. Ground it in what the input says of this game, such as the enemies' \
items and the score. Never a tip that fits any game, such as to play safe, ward more or \
watch the map. When no idea would change what the player does next, `advice` is null: \
silence beats a weak idea. `next_item` and `reason` are null unless the input lists Item \
candidates, `swap` is null unless it lists Swap candidates, and `elixir` is null unless \
it lists Elixir options. ";

pub const OPENAI_URL: &str = "https://api.openai.com";

// Fixed in code: the cost line in the Advice tab is measured on this model.
const MODEL: &str = "gpt-6-luna";
const USER_AGENT: &str = concat!("lollipop/", env!("CARGO_PKG_VERSION"));
