use std::mem::size_of;
use std::ops::{Index, IndexMut};

/// Intrusive slab with GPU buffer for use on GPU.
///
/// This is a simplified slab that doesn't track occupied/unoccupied slots, which is fine
/// if we only need to refer to elements by index and we never have to iterate.
///
/// Items must implement `GpuSlabItem` to store the free list pointer when a slot is free.
pub(crate) struct GpuSlab<T: GpuSlabItem + Copy> {
    items: Vec<T>,
    first_free: Option<usize>,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    label: String,
    dirty: bool,
}

pub(crate) trait GpuSlabItem {
    /// Index of next free item in the free list.
    fn next_free(&self) -> Option<usize>;
    /// Set the index of next free item in the free list.
    fn set_next_free(&mut self, i: Option<usize>);
}

impl<T: GpuSlabItem + Copy> GpuSlab<T> {
    /// Create a new `GpuSlab` with at least the specified capacity.
    pub fn new(device: &wgpu::Device, capacity: usize, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (size_of::<T>() * capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            items: Vec::with_capacity(capacity),
            first_free: None,
            buffer,
            buffer_capacity: capacity,
            label: label.to_string(),
            dirty: false,
        }
    }

    /// Insert an item in the slab and return an index into it.
    ///
    /// The index is stable and guaranteed to be valid until [`GpuSlab::remove()`] is called on it.
    pub fn insert(&mut self, item: T) -> usize {
        self.dirty = true;
        if let Some(first) = self.first_free {
            let next = self.items[first].next_free();
            self.first_free = next;
            self.items[first] = item;
            first
        } else {
            let idx = self.items.len();
            self.items.push(item);
            idx
        }
    }

    /// Remove an item.
    ///
    /// Removing an already-removed item will either panic or cause incorrect behavior.
    pub fn remove(&mut self, i: usize) {
        self.dirty = true;
        let item = &mut self.items[i];
        let next = self.first_free;
        item.set_next_free(next);
        self.first_free = Some(i);
    }

    /// Updates the underlying GPU buffer with current data.
    /// Returns true if the buffer was reallocated.
    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if !self.dirty {
            return false;
        }

        let should_realloc = self.items.len() > self.buffer_capacity;
        if should_realloc {
            self.buffer_capacity = self.items.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (size_of::<T>() * self.buffer_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !self.items.is_empty() {
            let size = self.items.len() * size_of::<T>();
            queue.write_buffer(&self.buffer, 0, unsafe {
                std::slice::from_raw_parts(self.items[..].as_ptr() as *const u8, size)
            });
        }

        self.dirty = false;
        should_realloc
    }

    pub fn bind_group_layout_entry(binding_index: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding_index,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(std::num::NonZeroU64::new(size_of::<T>() as u64).unwrap()),
            },
            count: None,
        }
    }

    pub fn bind_group_entry(&self, binding_index: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: binding_index,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: None,
            }),
        }
    }
}

impl<T: GpuSlabItem + Copy> Index<usize> for GpuSlab<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T: GpuSlabItem + Copy> IndexMut<usize> for GpuSlab<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.dirty = true;
        &mut self.items[index]
    }
}
