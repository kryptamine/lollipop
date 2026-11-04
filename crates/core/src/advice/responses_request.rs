//! The body of `POST /v1/responses`.

use serde::Serialize;
use serde_json::{json, Value};

use super::MODEL;

// The Widget's length cap in characters, which the answer is checked against.
pub(super) const ADVICE_LENGTH: usize = 120;
const OUTPUT_TOKENS: u32 = 120;

#[derive(Serialize)]
pub(super) struct ResponsesRequest<'a> {
    model: &'static str,
    store: bool,
    reasoning: Reasoning,
    max_output_tokens: u32,
    instructions: &'a str,
    input: &'a str,
    text: Text,
}

#[derive(Serialize)]
struct Reasoning {
    effort: &'static str,
}

#[derive(Serialize)]
struct Text {
    format: Value,
}

impl<'a> ResponsesRequest<'a> {
    pub(super) fn new(instructions: &'a str, input: &'a str) -> Self {
        Self {
            model: MODEL,
            store: false,
            reasoning: Reasoning { effort: "none" },
            max_output_tokens: OUTPUT_TOKENS,
            instructions,
            input,
            text: Text { format: format() },
        }
    }
}

// The same on every call: a schema OpenAI has not seen costs the first request extra latency.
fn format() -> Value {
    json!({
        "type": "json_schema",
        "name": "advice",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "advice": { "type": ["string", "null"] },
            },
            "required": ["advice"],
            "additionalProperties": false,
        },
    })
}
