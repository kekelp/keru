use std::ops::{Deref, DerefMut};

pub struct Observer<T> {
    value: T,
    timestamp: u64,
}

impl<T> Deref for Observer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        return &self.value
    }
}
impl<T> DerefMut for Observer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.timestamp = 0;
        return &mut self.value
    }
}

impl<T> Observer<T> {
    pub(crate) fn observe_changed(&mut self, current_frame: u64) -> bool {
        // if self.timestamp is 0, then changed = false
        // if many readers observe the value in the same frame, the first sets self.timestamp = current_frame, and all the others still see it as changed
        let changed = self.timestamp <= current_frame;
        
        self.timestamp = current_frame;

        return changed;
    }
}


