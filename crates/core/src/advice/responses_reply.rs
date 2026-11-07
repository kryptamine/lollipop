//! The `POST /v1/responses` reply around the model's own JSON.

use serde::Deserialize;

use super::failure::Failure;

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

/// The text the model wrote, which a refusal or any other reply lacks.
pub(super) fn output_text(body: &str) -> Result<String, Failure> {
    let response: Response = serde_json::from_str(body).map_err(|_| Failure::Schema)?;
    response
        .output
        .into_iter()
        .filter(|item| item.kind == "message")
        .flat_map(|item| item.content)
        .find(|content| content.kind == "output_text")
        .and_then(|content| content.text)
        .ok_or(Failure::Schema)
}
