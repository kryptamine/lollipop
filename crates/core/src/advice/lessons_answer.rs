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

/// The Lessons with blanks left out. Too many, or one over the length, fails the whole
/// answer: none is ever cut short.
pub(super) fn read_lessons(body: &str) -> Result<Vec<String>, Failure> {
    let text = output_text(body)?;
    let answer: LessonsAnswer = serde_json::from_str(&text).map_err(|_| Failure::Schema)?;
    let lessons: Vec<String> = answer
        .lessons
        .into_iter()
        .map(|lesson| lesson.trim().to_string())
        .filter(|lesson| !lesson.is_empty())
        .collect();
    if lessons.len() > LESSONS
        || lessons
            .iter()
            .any(|lesson| lesson.chars().count() > LESSON_LENGTH)
    {
        return Err(Failure::Length);
    }
    Ok(lessons)
}
