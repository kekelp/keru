mod tests;

use std::mem;
use bytemuck::Pod;

pub enum Metadata {
    Filled,
    Vacant { next_free: usize },
}

/// The user will also have to implement part of this trait in his shader code.
/// For example, the shader will skip rendering all entries that would return PodSlabMetadata::Vacant.
pub trait Entry: Pod {
    fn metadata(&self) -> Metadata;
    fn set_metadata(&mut self, metadata: Metadata);
}

fn dummy_vacant_entry<T: Entry>(next_free: usize) -> T {
    let mut value = T::zeroed();
    value.set_metadata(Metadata::Vacant { next_free });
    value
}

pub struct PodSlab<T: Entry> {
    entries: Vec<T>,
    first_free: usize,
    filled_count: usize,
}

impl<T: Entry> PodSlab<T> {
    fn push_filled_entry(&mut self, mut value: T) {
        value.set_metadata(Metadata::Filled);
        self.entries.push(value);
    }

    fn set_filled_entry(&mut self, i: usize, mut value: T) {
        value.set_metadata(Metadata::Filled);
        self.entries[i] = value;
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            first_free: 0,
            filled_count: 0,
        }
    }

    pub fn insert(&mut self, val: T) -> usize {
        self.filled_count += 1;

        // grab head of the freelist
        let key = self.first_free;

        if key == self.entries.len() {
            self.push_filled_entry(val);
            self.first_free = key + 1;
        } else {
            // set the new freelist head
            self.first_free = match self.entries[key].metadata() {
                Metadata::Vacant { next_free } => next_free,
                Metadata::Filled => unreachable!(),
            };
            // write the value
            self.set_filled_entry(key, val);
        }

        key
    }

    pub fn try_remove(&mut self, key: usize) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            let dummy_value = dummy_vacant_entry(self.first_free);
            let prev = mem::replace(entry, dummy_value);

            match prev.metadata() {
                Metadata::Filled => {
                    self.filled_count -= 1;
                    self.first_free = key;
                    return Some(prev);
                }
                _ => {
                    // If the previous entry was already vacant, then we restore the previous vacant entry.
                    // I copied this way of writing it from the `Slab` crate.
                    *entry = prev;
                }
            }
        }
        None
    }

    pub fn remove_or_panic(&mut self, key: usize) -> T {
        self.try_remove(key).expect("invalid key")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.entries
            .get(index)
            .and_then(|element| match element.metadata() {
                Metadata::Filled => Some(element),
                Metadata::Vacant { .. } => None,
            })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.entries
            .get_mut(index)
            .and_then(|element| match element.metadata() {
                Metadata::Filled => Some(element),
                Metadata::Vacant { .. } => None,
            })
    }

    /// Iterates over all filled entries
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries
            .iter()
            .filter(|element| matches!(element.metadata(), Metadata::Filled))
    }

    /// Iterates over all filled entries mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries
            .iter_mut()
            .filter(|element| matches!(element.metadata(), Metadata::Filled))
    }

    /// Iterates over all entries mutably, including vacant entries.
    pub fn iter_all_entries(&self) -> impl Iterator<Item = &T> {
        self.entries.iter()
    }

    /// Iterates over all entries mutably, including vacant entries.
    pub fn iter_all_entries_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries.iter_mut()
    }

    /// Get a raw slice of all entries, including the vacant ones.
    pub fn as_slice(&self) -> &[T] {
        &self.entries
    }
}
