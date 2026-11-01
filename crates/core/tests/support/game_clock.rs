use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lollipop_core::live::Clock;

/// Game time in seconds, moved only by the poll loop's own sleeps.
#[derive(Clone)]
pub struct GameClock(Arc<Mutex<f64>>);

impl GameClock {
    pub fn starting_at(game_time: f64) -> Self {
        Self(Arc::new(Mutex::new(game_time)))
    }

    pub fn now(&self) -> f64 {
        *self.0.lock().unwrap()
    }
}

impl Clock for GameClock {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        *self.0.lock().unwrap() += duration.as_secs_f64();
        tokio::task::yield_now()
    }
}
