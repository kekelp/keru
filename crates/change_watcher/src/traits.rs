use std::{fmt, hash::{Hash, Hasher}};
use crate::Watcher;

impl<T: Default> Default for Watcher<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PartialEq> PartialEq<T> for Watcher<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: PartialEq> PartialEq for Watcher<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Watcher<T> {}

impl<T: Hash> Hash for Watcher<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: Clone> Clone for Watcher<T> {
    fn clone(&self) -> Self {
        Watcher {
            value: self.value.clone(),
            // A cloned watcher will probably be read by a different reader, so it's probably more appropriate to set `changed` = `true`.
            changed: true,
        }
    }
}

impl<T> From<T> for Watcher<T> {
    fn from(value: T) -> Self {
        Watcher::new(value)
    }
}

impl<T: fmt::Display> fmt::Display for Watcher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
