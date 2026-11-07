/// Where the last Recap's window stands, which opens once per Recap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecapWindow {
    /// No Recap to open, or a new game has started.
    Shut,
    /// Made, waiting for the post-game screens.
    Awaiting,
    /// To open on this poll.
    Due,
    Opened,
}
