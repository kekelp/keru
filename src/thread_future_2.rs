use std::sync::mpsc::{channel, Receiver};
use std::task::Poll;
use std::thread;

/// A struct that will contain a `T` after a background computation finishes.
/// 
/// Obtained by calling [`run_in_background()`].
/// 
/// Use [`Self::poll()`] to check for the result.
pub struct ThreadFuture<T: Send + 'static>(Receiver<T>);

impl<T: Send + 'static> ThreadFuture<T> {
    /// Returns [`Poll::Pending`] if the value is not ready, or [`Poll::Ready(val)`] if the background function has finished executing. `val` is the owned result of the background function.
    pub fn poll(&self) -> Poll<T> {
        match self.0.try_recv() {
            Ok(value) => Poll::Ready(value),
            Err(_) => Poll::Pending,
        }
    }
    
    /// Blocks until the value is ready and returns it.
    pub fn wait(self) -> T {
        self.0.recv().expect("background thread panicked")
    }
}


/// A simple convenience function to compute a value in background.
/// 
/// This function spawns a background thread that will run `function`, and immediately returns a `ThreadFuture`.
/// 
/// When the function completes, its result will be stored in the `ThreadFuture`, and the `waker` function will be called.
/// 
/// As an example, the `waker` function can be used to unpause a `winit` event loop by calling `request_redraw` on an `Arc<Window>`.
pub fn run_in_background<T: Send + 'static>(
    function: impl FnOnce() -> T + Send + 'static,
    waker: impl FnOnce() + Send + 'static,
) -> ThreadFuture<T> {
    let (tx, rx) = channel();
    
    thread::spawn(move || {
        let result = function();
        let _ = tx.send(result);
        waker();
    });

    ThreadFuture(rx)
}