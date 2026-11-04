use std::future::Future;

use tokio::task::JoinHandle;

/// An Advisor call in flight. A dropped tokio task detaches rather than stops, so
/// dropping this, or assigning over it, aborts the call instead.
pub(super) struct PendingAnswer<T>(Option<JoinHandle<T>>);

impl<T> Default for PendingAnswer<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Send + 'static> PendingAnswer<T> {
    pub(super) fn spawn(call: impl Future<Output = T> + Send + 'static) -> Self {
        Self(Some(tokio::spawn(call)))
    }
}

impl<T> PendingAnswer<T> {
    /// Takes the answer once the call has finished, without waiting for it.
    pub(super) async fn finished(&mut self) -> Option<T> {
        self.0.take_if(|call| call.is_finished())?.await.ok()
    }
}

impl<T> Drop for PendingAnswer<T> {
    fn drop(&mut self) {
        if let Some(call) = self.0.take() {
            call.abort();
        }
    }
}
