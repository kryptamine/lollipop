//! The `POST /v1/responses` reply, and the model's JSON inside it.

use serde::Deserialize;

use super::responses_request::ADVICE_LENGTH;

/// What the model wrote, in the schema and within the Widget's length.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Answer {
    // `Option::deserialize` makes a missing field an error, where serde would read it as null.
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) advice: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) item: Option<u32>,
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
    /// `None` for anything outside the schema, a refusal, or advice over the length:
    /// the whole Advice is dropped, never cut short.
    pub(super) fn read(body: &str) -> Option<Self> {
        let response: Response = serde_json::from_str(body).ok()?;
        let text = response
            .output
            .iter()
            .filter(|item| item.kind == "message")
            .flat_map(|item| &item.content)
            .find(|content| content.kind == "output_text")?
            .text
            .as_deref()?;
        let answer: Self = serde_json::from_str(text).ok()?;
        let too_long = answer
            .advice
            .as_ref()
            .is_some_and(|advice| advice.chars().count() > ADVICE_LENGTH);
        (!too_long).then_some(answer)
    }
}
