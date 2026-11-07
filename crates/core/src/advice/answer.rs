//! The model's JSON inside a Trigger's reply.

use serde::Deserialize;

use super::failure::Failure;
use super::responses_reply::output_text;
use super::responses_request::ADVICE_LENGTH;

/// What the model wrote, in the schema and within the Widget's length.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Answer {
    // `Option::deserialize` makes a missing field an error, where serde would read it as null.
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) advice: Option<String>,
    /// An Item candidate to put in place of the Next item, `reason` saying why.
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) next_item: Option<u32>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) reason: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) swap: Option<SwapAnswer>,
    /// The id of the Elixir to drink before Baron and Elder.
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) elixir: Option<u32>,
}

/// An item to sell for an Item candidate once the inventory is full, and why.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwapAnswer {
    pub(crate) sell: u32,
    pub(crate) buy: u32,
    pub(crate) reason: String,
}

impl Answer {
    /// Fails for anything outside the schema, a refusal, or any text over the length:
    /// the whole Advice is dropped, never cut short.
    pub(super) fn read(body: &str) -> Result<Self, Failure> {
        let text = output_text(body)?;
        let answer: Self = serde_json::from_str(&text).map_err(|_| Failure::Schema)?;
        let swap_reason = answer.swap.as_ref().map(|swap| &swap.reason);
        let too_long = [answer.advice.as_ref(), answer.reason.as_ref(), swap_reason]
            .into_iter()
            .flatten()
            .any(|text| text.chars().count() > ADVICE_LENGTH);
        if too_long {
            return Err(Failure::Length);
        }
        Ok(answer)
    }
}
