use std::future::Future;
use std::time::Duration;

/// Paces the poll loop, so a replay can run on `gameTime` rather than real time.
pub trait Clock: Sync {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }
}
