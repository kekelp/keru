use bytemuck::Pod;

pub enum PodSlabMetadata {
    Filled,
    Vacant {
        next_free: usize,
    }
}

/// The user will also have to implement part of this trait in his shader code.
/// For example, the shader will skip rendering all elements that would return PodSlabMetadata::Vacant.
pub trait PodSlabElement: Pod {
    fn metadata(&self) -> PodSlabMetadata;
    fn set_metadata(&mut self, metadata: PodSlabMetadata);
}

pub struct PodSlab<T: PodSlabElement> {
    data: Vec<T>,
    next: usize,
}

impl<T: PodSlabElement> PodSlab<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next: 0,
        }
    }

    pub fn insert(&mut self, mut element: T) -> usize {
        // If we have no free slots, append to the end
        if self.next >= self.data.len() {
            element.set_metadata(PodSlabMetadata::Filled);
            self.data.push(element);
            return self.data.len() - 1;
        }

        // Otherwise, use the first free slot
        let index = self.next;
        
        // Get the next free slot from the current one's metadata
        if let PodSlabMetadata::Vacant { next_free } = self.data[index].metadata() {
            self.next = next_free;
        } else {
            panic!("Corrupted free list: expected vacant slot");
        }

        // Place the new element
        element.set_metadata(PodSlabMetadata::Filled);
        self.data[index] = element;
        
        index
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.data.len() {
            return None;
        }

        // Check if the slot is actually filled
        if let PodSlabMetadata::Vacant { .. } = self.data[index].metadata() {
            return None;
        }

        // Mark the slot as vacant and update the free list
        let mut element = std::mem::replace(&mut self.data[index], unsafe { std::mem::zeroed() });
        self.data[index].set_metadata(PodSlabMetadata::Vacant { next_free: self.next });
        self.next = index;

        // Clear the element's metadata before returning
        element.set_metadata(PodSlabMetadata::Vacant { next_free: 0 });
        Some(element)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index).and_then(|element| {
            match element.metadata() {
                PodSlabMetadata::Filled => Some(element),
                PodSlabMetadata::Vacant { .. } => None,
            }
        })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index).and_then(|element| {
            match element.metadata() {
                PodSlabMetadata::Filled => Some(element),
                PodSlabMetadata::Vacant { .. } => None,
            }
        })
    }

    /// Iterates over all filled elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter(|element| {
            matches!(element.metadata(), PodSlabMetadata::Filled)
        })
    }

    /// Iterates over all filled elements mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().filter(|element| {
            matches!(element.metadata(), PodSlabMetadata::Filled)
        })
    }

    /// For direct GPU upload - includes vacant elements
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;
    
    // Start with a custom Pod datastructure that we want a slab of.
    // For Pod friendliness and ease of use, we stick the metadata directly into the struct.
    // Since this is all done manually by the user of the library, they can still get creative with layout and bit-packing, if they want.
    // The user will also manually write their shaders to be aware of this metadata, so that they can skip rendering vacant entries.
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
    impl PodSlabElement for Vertex {
        fn metadata(&self) -> PodSlabMetadata {
            if self.filled == 0 {
                return PodSlabMetadata::Vacant { next_free: self.next_free };
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
                },
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
        let mut slab = PodSlab::new();
        let v1 = create_test_vertex(1.0);
        let v2 = create_test_vertex(2.0);

        let idx1 = slab.insert(v1);
        let idx2 = slab.insert(v2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(slab.get(idx1).unwrap().position[0], 1.0);
        assert_eq!(slab.get(idx2).unwrap().position[0], 2.0);
    }
}