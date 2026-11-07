//! What the Recap window may ask of the app.
#![allow(
    clippy::needless_pass_by_value,
    reason = "a Tauri command receives its arguments by value"
)]

use lollipop_core::live::Recap;
use tauri::State;

use crate::last_recap::LastRecap;

#[tauri::command]
pub(crate) fn read_recap(last: State<'_, LastRecap>) -> Option<Recap> {
    last.get()
}
