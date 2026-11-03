use crate::advice::Trigger;

/// An answer a Trigger has for the Advice Widget.
pub(super) struct Arrival {
    pub(super) trigger: Trigger,
    pub(super) text: String,
    pub(super) icon: Option<String>,
}
