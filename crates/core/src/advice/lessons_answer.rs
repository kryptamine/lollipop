//! The model's JSON inside a Recap's reply.

use serde::Deserialize;

use super::failure::Failure;
use super::responses_reply::output_text;
use super::responses_request::{LESSONS, LESSON_LENGTH};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonsAnswer {
    lessons: Vec<String>,
}

/// The Lessons with blanks left out. Too many fails the whole answer; one over the length
/// is shortened at a word, since models count characters poorly.
pub(super) fn read_lessons(body: &str) -> Result<Vec<String>, Failure> {
    let text = output_text(body)?;
    let answer: LessonsAnswer = serde_json::from_str(&text).map_err(|_| Failure::Schema)?;
    let lessons: Vec<String> = answer
        .lessons
        .into_iter()
        .map(|lesson| lesson.trim().to_string())
        .filter(|lesson| !lesson.is_empty())
        .collect();
    if lessons.len() > LESSONS {
        return Err(Failure::Length);
    }
    Ok(lessons.iter().map(|lesson| shortened(lesson)).collect())
}

/// At most `LESSON_LENGTH` characters, cut at the last whole word and ending in `…`.
fn shortened(lesson: &str) -> String {
    if lesson.chars().count() <= LESSON_LENGTH {
        return lesson.to_string();
    }
    let kept: String = lesson.chars().take(LESSON_LENGTH - 1).collect();
    let words = kept
        .rfind(char::is_whitespace)
        .map_or(kept.as_str(), |space| &kept[..space]);
    let end = words.trim_end_matches(|c: char| c.is_whitespace() || c.is_ascii_punctuation());
    format!("{end}…")
}
