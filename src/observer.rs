use std::sync::atomic::{AtomicU64, Ordering};
use std::ops::{Deref, DerefMut};

// Global counter for fake timestamps
static FAKE_TIME: AtomicU64 = AtomicU64::new(0);

pub struct Observer<T> {
    value: T,
    changed_at: u64, // Stores the last counter value when the value was modified
}

impl<T> Observer<T> {
    pub fn new(value: T) -> Self {
        Observer {
            value,
            changed_at: FAKE_TIME.load(Ordering::Relaxed),
        }
    }
}

impl<T> Deref for Observer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Observer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Increment the global counter and store the new value
        self.changed_at = FAKE_TIME.fetch_add(1, Ordering::SeqCst);
        &mut self.value
    }
}

impl<T> Observer<T> {
    pub fn observe_changes(&self, last_frame_end: u64) -> bool {
        // Check if the value was modified after the renderer's last frame
        last_frame_end < self.changed_at
    }
    pub fn changed_at(&self) -> u64 {
        self.changed_at
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct TestUi {
        last_frame_end: u64,
    }

    impl TestUi {
        fn new() -> Self {
            TestUi {
                last_frame_end: FAKE_TIME.fetch_add(1, Ordering::SeqCst),
            }
        }

        fn advance_frame(&mut self) {
            // Simulate advancing a frame by updating the last_frame_end to the current counter value
            self.last_frame_end = FAKE_TIME.fetch_add(1, Ordering::SeqCst);
        }
    }


    #[test]
    fn test_observer() {
        let mut observer = Observer::new(17);
        let mut renderer1 = TestUi::new();
        let mut renderer2 = TestUi::new();

        assert!(!observer.observe_changes(renderer1.last_frame_end));
        assert!(!observer.observe_changes(renderer2.last_frame_end));

        *observer += 123;

        assert!(observer.observe_changes(renderer1.last_frame_end));
        assert!(observer.observe_changes(renderer2.last_frame_end));

        renderer1.advance_frame();

        assert!(!observer.observe_changes(renderer1.last_frame_end));

        assert!(observer.observe_changes(renderer2.last_frame_end));

        renderer2.advance_frame();

        assert!(!observer.observe_changes(renderer1.last_frame_end));
        assert!(!observer.observe_changes(renderer2.last_frame_end));

        *observer += 1;

        assert!(observer.observe_changes(renderer1.last_frame_end));
        assert!(observer.observe_changes(renderer2.last_frame_end));
    }
}