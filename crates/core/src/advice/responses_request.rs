//! The body of `POST /v1/responses`.

use serde::Serialize;
use serde_json::{json, Value};

use super::MODEL;

// The Widget's length cap in characters, which every text of the answer is checked against.
pub(super) const ADVICE_LENGTH: usize = 120;
// Room for the three texts at their cap.
const OUTPUT_TOKENS: u32 = 360;
/// The most Lessons a Recap shows, and the length each is checked against.
pub(super) const LESSONS: usize = 3;
pub(super) const LESSON_LENGTH: usize = 160;
const LESSON_TOKENS: u32 = 480;

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
        Self::with_format(instructions, input, OUTPUT_TOKENS, format())
    }

    pub(super) fn for_lessons(instructions: &'a str, input: &'a str) -> Self {
        Self::with_format(instructions, input, LESSON_TOKENS, lessons_format())
    }

    const fn with_format(
        instructions: &'a str,
        input: &'a str,
        max_output_tokens: u32,
        format: Value,
    ) -> Self {
        Self {
            model: MODEL,
            store: false,
            reasoning: Reasoning { effort: "none" },
            max_output_tokens,
            instructions,
            input,
            text: Text { format },
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
                "next_item": { "type": ["integer", "null"] },
                "reason": { "type": ["string", "null"] },
                "swap": {
                    "anyOf": [
                        {
                            "type": "object",
                            "properties": {
                                "sell": { "type": "integer" },
                                "buy": { "type": "integer" },
                                "reason": { "type": "string" },
                            },
                            "required": ["sell", "buy", "reason"],
                            "additionalProperties": false,
                        },
                        { "type": "null" },
                    ],
                },
                "elixir": { "type": ["integer", "null"] },
            },
            "required": ["advice", "next_item", "reason", "swap", "elixir"],
            "additionalProperties": false,
        },
    })
}

fn lessons_format() -> Value {
    json!({
        "type": "json_schema",
        "name": "lessons",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "lessons": { "type": "array", "items": { "type": "string" } },
            },
            "required": ["lessons"],
            "additionalProperties": false,
        },
    })
}
