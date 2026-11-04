// About 10-18 s at one poll a second with its 800 ms timeout. A loading screen left
// without a game is followed by champ select, minutes before the next one.
const FAILED_POLLS_TO_LEAVE: u32 = 10;

/// Keeps a loading screen through polls League fails to answer, there and as the
/// game takes over from it.
#[derive(Default)]
pub(super) struct HeldLoadingScreen {
    /// `None` while no loading screen is held.
    failed_polls: Option<u32>,
}

impl HeldLoadingScreen {
    pub(super) const fn seen(&mut self) {
        self.failed_polls = Some(0);
    }

    pub(super) const fn end(&mut self) {
        self.failed_polls = None;
    }

    /// Whether the loading screen still holds after one more failed poll.
    pub(super) const fn failed_poll(&mut self) -> bool {
        let Some(failed) = self.failed_polls.as_mut() else {
            return false;
        };
        *failed += 1;
        if *failed >= FAILED_POLLS_TO_LEAVE {
            self.end();
            return false;
        }
        true
    }
}
