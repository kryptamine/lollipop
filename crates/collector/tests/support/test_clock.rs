use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use lollipop_collector::Clock;

const START_MS: u64 = 1_800_000_000_000;

/// Moved only by the collector's own sleeps, so a test runs rate limit windows at once.
#[derive(Clone)]
pub struct TestClock {
    now_ms: Arc<AtomicU64>,
    frozen: Arc<AtomicBool>,
}

impl TestClock {
    pub fn new() -> Self {
        Self {
            now_ms: Arc::new(AtomicU64::new(START_MS)),
            frozen: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn advance(&self, duration: Duration) {
        let millis = u64::try_from(duration.as_millis()).unwrap();
        self.now_ms.fetch_add(millis, Ordering::SeqCst);
    }

    /// Parks every sleep where it is, so a test can end a run in the middle of a wait.
    pub fn set_frozen(&self, frozen: bool) {
        self.frozen.store(frozen, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }

    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        let clock = self.clone();
        async move {
            while clock.frozen.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            clock.advance(duration);
            tokio::task::yield_now().await;
        }
    }
}
