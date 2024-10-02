mod traits;
mod tests;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Watcher<T> {
    value: T,
    changed: bool,
}

impl<T> Deref for Watcher<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Watcher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.value
    }
}

impl<T> AsRef<T> for Watcher<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> AsMut<T> for Watcher<T> {
    fn as_mut(&mut self) -> &mut T {
        self.changed = true;
        &mut self.value
    }
}

impl <T> Watcher<T> {
    /// Initialize a new `Watcher`, which starts out as "changed" / "not synced".
    pub fn new(value: T) -> Self {
        Self {
            value,
            // If the value has just been created, the reader definitely won't be in sync with it, so `synced` starts as `false`. 
            changed: true,
        }
    }

    /// Read the value, only if it has changed, and mark the watcher as synced.
    /// Subsequent calls to `if_changed()` will return `None` until the value is changed again.
    pub fn if_changed(&mut self) -> Option<&T> {
        match self.changed {
            false => None,
            true => {
                self.changed = false;
                Some(&self.value)
            },
        }
    }
}
