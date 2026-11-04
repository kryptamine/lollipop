use std::future::Future;
use std::pin::Pin;

use super::build::Build;
use super::until::Until;
use crate::advice::Language;

pub(super) type InputFuture = Pin<Box<dyn Future<Output = String> + Send>>;

/// What a Trigger saw at this poll that changes its Advice.
pub(super) enum Fired<Input> {
    /// `input` reads the Build once its lead is tracked; its future runs with the call.
    Ask { until: Until, input: Input },
    /// The answer on its way no longer fits.
    Drop,
    /// Drops the answer on its way, and asks nothing for the reason given.
    Skip(&'static str),
}

/// What a Trigger hands back when it fires an ask.
pub(super) trait AskInput: FnOnce(Language, &Build) -> InputFuture {}

impl<F: FnOnce(Language, &Build) -> InputFuture> AskInput for F {}
