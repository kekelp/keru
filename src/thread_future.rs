use std::{sync::Arc, thread};
use std::sync::OnceLock;
use std::task::Poll;

/// A struct that will contain a `T` after a background computation finishes.
/// 
/// Obtained by calling [`run_in_background()`].
/// 
/// Use [`Self::poll()`] to extract the result, when it's ready.
pub struct ThreadFuture<T: Send + Sync + 'static>(Arc<OnceLock<T>>);

impl<T: Send + Sync + 'static> Clone for ThreadFuture<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// A simple convenience function to compute a value in background.
/// 
/// This function spawns a background thread that will run `function`, and immediately returns a `ThreadFuture`.
/// 
/// When the function completes, its result will be stored in the `ThreadFuture`, and the `waker` function will be called.
/// 
/// As an example, the `waker` function can be used to unpause a `winit` event loop by calling `request_redraw` on an `Arc<Window>`.
/// 
/// See the `async_thread.rs` example.
pub fn run_in_background<T: Send + Sync + 'static>(
    function: impl FnOnce() -> T + Send + 'static,
    waker: impl FnOnce() + Send + 'static,
) -> ThreadFuture<T> {
    let arc = Arc::new(OnceLock::new());
    let clone = Arc::clone(&arc);
    
    thread::spawn(move || {
        let result = function();
        let _ = clone.set(result);
        waker();
    });

    return ThreadFuture(arc);
}

impl<T: Send + Sync + 'static> ThreadFuture<T> {
    /// Returns [`Poll::Pending`] if the value is not ready, or [`Poll::Ready(val)`] if the background function has finished executing. `val` is a reference to the result of the background function.
    pub fn poll(&self) -> Poll<&T> {
        match self.0.get() {
            Some(value) => Poll::Ready(&value),
            None => Poll::Pending,
        }
    }
}
