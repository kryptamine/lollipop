//! The `POST /v1/responses` reply, and the model's JSON inside it.

use serde::Deserialize;

use super::failure::Failure;
use super::responses_request::ADVICE_LENGTH;

/// What the model wrote, in the schema and within the Widget's length.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Answer {
    // `Option::deserialize` makes a missing field an error, where serde would read it as null.
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) advice: Option<String>,
}

#[derive(Deserialize)]
struct Response {
    output: Vec<OutputItem>,
}

#[derive(Deserialize)]
struct OutputItem {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

impl Answer {
    /// Fails for anything outside the schema, a refusal, or advice over the length:
    /// the whole Advice is dropped, never cut short.
    pub(super) fn read(body: &str) -> Result<Self, Failure> {
        let response: Response = serde_json::from_str(body).map_err(|_| Failure::Schema)?;
        let text = response
            .output
            .iter()
            .filter(|item| item.kind == "message")
            .flat_map(|item| &item.content)
            .find(|content| content.kind == "output_text")
            .and_then(|content| content.text.as_deref())
            .ok_or(Failure::Schema)?;
        let answer: Self = serde_json::from_str(text).map_err(|_| Failure::Schema)?;
        let too_long = answer
            .advice
            .as_ref()
            .is_some_and(|advice| advice.chars().count() > ADVICE_LENGTH);
        if too_long {
            return Err(Failure::Length);
        }
        Ok(answer)
    }
}
