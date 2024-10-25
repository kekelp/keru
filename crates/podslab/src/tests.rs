#[cfg(test)]

use bytemuck::Pod;
use crate::*;

mod slab_vertex {
    use super::*;
    use bytemuck::Zeroable;

    
    // Start with a custom Pod datastructure that we want a slab of.
    // We will insert some metadata in addition to the real data. The metadata will have to be enough to implement the Entry trait below.
    // That is, it has to be able to hold a representation of the PodSlab::Metadata struct.
    #[derive(Copy, Clone, Debug, Zeroable, Pod)]
    #[repr(C)]
    pub struct Vertex {
        pub position: [f32; 4],
        pub normal: [f32; 4],
        pub color: [f32; 4],
        // For the sake of memory usage, we stick all the slab metadata into a single u32.
        // We will use u32::MAX to denote a filled entry. Any other value means vacant, with the value being the freelist index.
        // This means that a PodSlab<Vertex> won't work properly if we push more than u32::MAX entries into it.
        // Since we are in control of the representation, we can make this compromise.
        // If we don't want to, we can just add some bytes and get a lossless representation.
        //
        // The slab will overwrite this value with the `set_metadata` function according to its internal logic.
        // If this field was public, it would be up to our own common sense to not reach inside the slab and change the metadata of the entries.
        // For this reason, it's probably wise to make it private.
        slab_metadata: u32,
    }

    // Tell PodSlab how the basic slab functions are implemented on top of the custom metadata.
    // If this implementation is inconsistent, the slab won't work properly. However, it's really simple.
    // We just need to convert the PodSlab::Metadata into any kind of 
    impl Entry for Vertex {
        fn metadata(&self) -> Metadata {
            if self.slab_metadata == u32::MAX {
                Metadata::Filled
            } else {
                Metadata::Vacant {
                    // casting the u32 to usize is probably fine, unless the architecture is weird AND the slab has a ton of entries.
                    next_free: self.slab_metadata as usize,
                }
            }
        }

        fn set_metadata(&mut self, metadata: Metadata) {
            match metadata {
                Metadata::Filled => self.slab_metadata = u32::MAX,
                Metadata::Vacant { next_free } => {
                    // casting the usize to u32 is probably fine, unless the slab has a ton of entries.
                    self.slab_metadata = next_free as u32;
                }
            }
        }
    }

    impl Vertex {
        // Since we made the `slab_metadata` field private, we won't be able to even construct the struct outside of this module.
        // So we have to provide a constructor method.
        // A valid alternative is to keep everything public and just use common sense.
        pub fn new(position: [f32; 4], normal: [f32; 4], color: [f32; 4]) -> Self {
            Self {
                position,
                normal,
                color,
                slab_metadata: 0, // Any value is fine, because the slab will overwrite it.
            }
        }
    }
}

use slab_vertex::Vertex;

#[allow(dead_code)]
fn create_test_vertex(pos: f32) -> Vertex {
    Vertex::new(
        [pos, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 0.0],
        [1.0, 1.0, 1.0, 1.0],
    )
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
