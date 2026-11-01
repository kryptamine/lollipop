use std::future::Future;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Paces calls to Riot, so a test can run a rate limit window without waiting for it.
pub trait Clock: Sync {
    /// Milliseconds since the Unix epoch. Rate limit windows outlive a restart on it.
    fn now_ms(&self) -> u64;
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

#[derive(Debug)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH);
        elapsed.map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
    }

    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }
}
