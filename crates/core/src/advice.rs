//! Advice from OpenAI, called with the player's own key: see ADR 0016.

mod key_check;
mod masked_key;

pub use key_check::{check_key, KeyCheck};
pub use masked_key::masked_key;

pub const OPENAI_URL: &str = "https://api.openai.com";

// Fixed in code: the cost line in the Advice tab is measured on this model.
const MODEL: &str = "gpt-6-luna";
