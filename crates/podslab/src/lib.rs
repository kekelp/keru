use std::mem;
use bytemuck::Pod;

pub enum PodSlabMetadata {
    Filled,
    Vacant { next_free: usize },
}

/// The user will also have to implement part of this trait in his shader code.
/// For example, the shader will skip rendering all entries that would return PodSlabMetadata::Vacant.
pub trait PodSlabEntry: Pod {
    fn metadata(&self) -> PodSlabMetadata;
    fn set_metadata(&mut self, metadata: PodSlabMetadata);
}

fn dummy_vacant_entry<T: PodSlabEntry>(next_free: usize) -> T {
    let mut value = T::zeroed();
    value.set_metadata(PodSlabMetadata::Vacant { next_free });
    value
}

pub struct PodSlab<T: PodSlabEntry> {
    entries: Vec<T>,
    next: usize,
    n_filled_entries: usize,
}

impl<T: PodSlabEntry> PodSlab<T> {
    fn push_filled_entry(&mut self, mut value: T) {
        value.set_metadata(PodSlabMetadata::Filled);
        self.entries.push(value);
    }

    fn set_filled_entry(&mut self, i: usize, mut value: T) {
        value.set_metadata(PodSlabMetadata::Filled);
        self.entries[i] = value;
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next: 0,
            n_filled_entries: 0,
        }
    }

    pub fn insert(&mut self, val: T) -> usize {
        self.n_filled_entries += 1;

        // grab head of the freelist
        let key = self.next;

        if key == self.entries.len() {
            self.push_filled_entry(val);
            self.next = key + 1;
        } else {
            // set the new freelist head
            self.next = match self.entries[key].metadata() {
                PodSlabMetadata::Vacant { next_free } => next_free,
                PodSlabMetadata::Filled => unreachable!(),
            };
            // write the value
            self.set_filled_entry(key, val);
        }

        key
    }

    pub fn try_remove(&mut self, key: usize) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            let dummy_value = dummy_vacant_entry(self.next);
            let prev = mem::replace(entry, dummy_value);

            match prev.metadata() {
                PodSlabMetadata::Filled => {
                    self.n_filled_entries -= 1;
                    self.next = key;
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
                PodSlabMetadata::Filled => Some(element),
                PodSlabMetadata::Vacant { .. } => None,
            })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.entries
            .get_mut(index)
            .and_then(|element| match element.metadata() {
                PodSlabMetadata::Filled => Some(element),
                PodSlabMetadata::Vacant { .. } => None,
            })
    }

    /// Iterates over all filled entries
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries
            .iter()
            .filter(|element| matches!(element.metadata(), PodSlabMetadata::Filled))
    }

    /// Iterates over all filled entries mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.entries
            .iter_mut()
            .filter(|element| matches!(element.metadata(), PodSlabMetadata::Filled))
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    // Start with a custom Pod datastructure that we want a slab of.
    // For Pod friendliness and ease of use, we stick the metadata directly into the struct.
    // Since this is all done manually by the user of the library, they can still get creative with layout and bit-packing, if they want.
    // The user will also have to write their shaders to be aware of this metadata, so that they can skip rendering vacant entries.
    #[derive(Copy, Clone, Debug, Zeroable, Pod)]
    #[repr(C)]
    struct Vertex {
        position: [f32; 4],
        normal: [f32; 4],
        color: [f32; 4],
        filled: usize,
        next_free: usize,
    }

    // Tell PodSlab how the basic slab functions are implemented on top of the custom metadata.
    impl PodSlabEntry for Vertex {
        fn metadata(&self) -> PodSlabMetadata {
            if self.filled == 0 {
                return PodSlabMetadata::Vacant {
                    next_free: self.next_free,
                };
            } else {
                return PodSlabMetadata::Filled;
            }
        }

        fn set_metadata(&mut self, metadata: PodSlabMetadata) {
            match metadata {
                PodSlabMetadata::Filled => self.filled = 1,
                PodSlabMetadata::Vacant { next_free } => {
                    self.filled = 0;
                    self.next_free = next_free;
                }
            }
        }
    }

    fn create_test_vertex(pos: f32) -> Vertex {
        Vertex {
            position: [pos, 0.0, 0.0, 1.0],
            normal: [0.0, 1.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            filled: 0,
            next_free: 0,
        }
    }

    #[test]
    fn test_basic_insertion() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);

        let idx1 = slab.insert(v1);
        let idx2 = slab.insert(v2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(slab.get(idx1).unwrap().position[0], 1.0);
        assert_eq!(slab.get(idx2).unwrap().position[0], 2.0);
    }

    #[test]
    fn test_insert_remove_reinsert() {
        let mut slab = PodSlab::with_capacity(16);

        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);

        // Insert entries
        let idx1 = slab.insert(v1);
        let idx2 = slab.insert(v2);

        // Check inserted values
        assert_eq!(slab.get(idx1).unwrap().position[0], 1.0);
        assert_eq!(slab.get(idx2).unwrap().position[0], 2.0);

        // Remove one element
        let removed = slab.try_remove(idx1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().position[0], 1.0);

        // The slot for `idx1` should be vacant now
        assert!(slab.get(idx1).is_none());

        // Reinsert a new element, it should reuse `idx1`
        let v3 = create_test_vertex(3.0);
        let idx3 = slab.insert(v3);
        assert_eq!(idx3, idx1); // idx3 should reuse the vacant slot at idx1
        assert_eq!(slab.get(idx3).unwrap().position[0], 3.0);
    }

    #[test]
    fn test_retrieve_vacant_slot() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        let idx1 = slab.insert(v1);

        // Remove the element, making it vacant
        slab.try_remove(idx1);

        // Attempt to get the element at the vacant slot
        assert!(slab.get(idx1).is_none());
        assert!(slab.get_mut(idx1).is_none());
    }

    #[test]
    fn test_iter() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);
        let v3 = create_test_vertex(3.0);

        // Insert entries
        slab.insert(v1);
        let idx2 = slab.insert(v2);
        slab.insert(v3);

        // Remove the second element
        slab.try_remove(idx2);

        // Only v1 and v3 should be iterated over, since v2 is vacant
        let positions: Vec<f32> = slab.iter().map(|v| v.position[0]).collect();
        assert_eq!(positions, vec![1.0, 3.0]);
    }

    #[test]
    fn test_iter_mut() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);

        // Insert entries
        let idx1 = slab.insert(v1);
        let idx2 = slab.insert(v2);

        // Modify values through `iter_mut`
        for vertex in slab.iter_mut() {
            vertex.position[0] += 1.0;
        }

        // Check modifications
        assert_eq!(slab.get(idx1).unwrap().position[0], 2.0);
        assert_eq!(slab.get(idx2).unwrap().position[0], 3.0);
    }

    #[test]
    fn test_out_of_bounds_removal() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        slab.insert(v1);

        // Attempting to remove an out-of-bounds index should return None
        assert!(slab.try_remove(10).is_none());
    }

    #[test]
    fn test_direct_gpu_upload_slice() {
        let mut slab = PodSlab::with_capacity(16);
        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);
        let v3 = create_test_vertex(3.0);
    
        // Insert entries
        slab.insert(v1);
        let idx2 = slab.insert(v2);
        slab.insert(v3);
    
        // Remove one element
        slab.remove_or_panic(idx2);
    
        // Ensure `as_slice` includes all entries, even vacant ones
        let all_entries = slab.as_slice();
        assert_eq!(all_entries.len(), 3);
    
        // Cast to u8 slice for GPU upload
        let byte_slice: &[u8] = bytemuck::cast_slice(all_entries);
        
        // Check that the byte slice length matches the expected size
        // Each Vertex is converted to u8, so length should be `3 * size_of::<Vertex>()`
        assert_eq!(byte_slice.len(), 3 * std::mem::size_of::<Vertex>());
    }
    
}
