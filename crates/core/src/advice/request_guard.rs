/// OpenAI requests a game may make, retries included: a runaway guard, not a setting.
pub(super) const REQUESTS_PER_GAME: u32 = 100;

/// What Advice remembers across calls.
#[derive(Debug, Default)]
pub(super) struct RequestGuard {
    requests: u32,
    said_unreachable: bool,
    /// OpenAI refused the key, so no call is made until a key is saved.
    key_failed: bool,
}

pub(super) enum Request {
    Granted,
    /// The first request past the cap, which the log reports.
    CapReached,
    Refused,
}

impl RequestGuard {
    pub(super) const fn take_request(&mut self) -> Request {
        self.requests = self.requests.saturating_add(1);
        if self.requests <= REQUESTS_PER_GAME {
            Request::Granted
        } else if self.requests == REQUESTS_PER_GAME + 1 {
            Request::CapReached
        } else {
            Request::Refused
        }
    }

    /// True only the first time in a game.
    pub(super) const fn say_unreachable(&mut self) -> bool {
        !std::mem::replace(&mut self.said_unreachable, true)
    }

    pub(super) const fn key_failed(&self) -> bool {
        self.key_failed
    }

    pub(super) const fn set_key_failed(&mut self, failed: bool) {
        self.key_failed = failed;
    }

    /// A refused key stays refused into the next game.
    pub(super) const fn end_game(&mut self) {
        self.requests = 0;
        self.said_unreachable = false;
    }
}
