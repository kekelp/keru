use std::mem::size_of;

use offset_allocator::{Allocation, Allocator};
use wgpu::util::StagingBelt;

pub const CHUNK_SIZE: u32 = 4096;

/// A handle to a live allocation in the heap.
///
/// The element index of this allocation within the backing `Vec` is
/// `chunk_index * chunk_size + allocation.offset`.
#[derive(Clone, Copy)]
pub struct Handle {
    pub chunk_index: u32,
    pub allocation: Allocation,
    pub size: u32,
}

impl Handle {
    /// The element index of this allocation within the backing `Vec`.
    pub fn vec_index(&self, chunk_size: u32) -> usize {
        (self.chunk_index * chunk_size + self.allocation.offset) as usize
    }
}

pub struct GpuHeap<T> {
    chunks: Vec<Allocator>,
    // Logical element count of the heap (chunks.len() * CHUNK_SIZE).
    vec_len: usize,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    label: String,
    usage: wgpu::BufferUsages,
    staging_belt: StagingBelt,
    dirty: bool,
    // Set whenever the GPU buffer is replaced so the caller can recreate bind groups.
    reallocated: bool,
    device: wgpu::Device,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Copy + Default + Clone> GpuHeap<T> {
    pub fn with_usage(device: &wgpu::Device, capacity: usize, label: &str, usage: wgpu::BufferUsages) -> Self {
        // COPY_SRC is needed so we can copy old contents into the replacement buffer on grow.
        let usage = usage | wgpu::BufferUsages::COPY_SRC;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (size_of::<T>() * capacity) as u64,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            buffer_capacity: capacity,
            chunks: Vec::with_capacity(1),
            vec_len: 0,
            label: label.to_string(),
            usage,
            staging_belt: StagingBelt::new(device.clone(), 16384),
            dirty: false,
            reallocated: false,
            device: device.clone(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn belt_write_bytes(&mut self, byte_offset: u64, bytes: &[u8], encoder: &mut wgpu::CommandEncoder) {
        let Some(size) = wgpu::BufferSize::new(bytes.len() as u64) else { return };
        let Self { staging_belt, buffer, .. } = self;
        let mut view = staging_belt.write_buffer(encoder, buffer, byte_offset, size);
        view.copy_from_slice(bytes);
        self.dirty = true;
    }

    /// If `vec_len` has grown past `buffer_capacity`, replaces the buffer and copies old data.
    fn maybe_grow_buffer(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.vec_len <= self.buffer_capacity {
            return;
        }

        let new_capacity = self.vec_len.next_power_of_two();
        let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label.as_str()),
            size: (size_of::<T>() * new_capacity) as u64,
            usage: self.usage,
            mapped_at_creation: false,
        });

        let old_bytes = (self.buffer_capacity * size_of::<T>()) as u64;
        if old_bytes > 0 {
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, old_bytes);
        }

        self.buffer = new_buffer;
        self.buffer_capacity = new_capacity;
        self.reallocated = true;
    }

    fn allocate(&mut self, size: u32, encoder: &mut wgpu::CommandEncoder) -> Option<Handle> {
        for (i, chunk) in self.chunks.iter_mut().enumerate() {
            if let Some(allocation) = chunk.allocate(size) {
                return Some(Handle { chunk_index: i as u32, allocation, size });
            }
        }

        if size > CHUNK_SIZE {
            return None;
        }

        let chunk_index = self.chunks.len() as u32;
        self.vec_len += CHUNK_SIZE as usize;
        let mut new_chunk = Allocator::with_max_allocs(CHUNK_SIZE, CHUNK_SIZE);
        let allocation = new_chunk.allocate(size).unwrap();
        self.chunks.push(new_chunk);

        self.maybe_grow_buffer(encoder);

        Some(Handle { chunk_index, allocation, size })
    }

    fn heap_free(&mut self, handle: Handle) {
        self.chunks[handle.chunk_index as usize].free(handle.allocation);
    }

    fn write_handle(&mut self, handle: Handle, data: &[T], encoder: &mut wgpu::CommandEncoder) {
        let byte_offset = (handle.vec_index(CHUNK_SIZE) * size_of::<T>()) as u64;
        let data_bytes = data.len() * size_of::<T>();
        let tail_bytes = (handle.size as usize - data.len()) * size_of::<T>();

        let data_u8 = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data_bytes) };
        self.belt_write_bytes(byte_offset, data_u8, encoder);

        if tail_bytes > 0 {
            let zeros = vec![0u8; tail_bytes];
            self.belt_write_bytes(byte_offset + data_bytes as u64, &zeros, encoder);
        }
    }

    fn zero_handle(&mut self, handle: Handle, encoder: &mut wgpu::CommandEncoder) {
        let byte_offset = (handle.vec_index(CHUNK_SIZE) * size_of::<T>()) as u64;
        let zeros = vec![0u8; handle.size as usize * size_of::<T>()];
        self.belt_write_bytes(byte_offset, &zeros, encoder);
    }

    pub fn free_and_clear(&mut self, handle: Handle, encoder: &mut wgpu::CommandEncoder) {
        self.zero_handle(handle, encoder);
        self.heap_free(handle);
    }

    /// If `handle` is `None`, allocates a new region with enough space for `data`, then updates `handle` to contain a handle to the new allocation, then writes `data` in the new region.
    ///
    /// If `handle` is `Some` and points to a region that's too small to contain `data`, clears and frees the region and allocates a new one, sets `handle` to point at the new one, then writes `data`.
    ///
    /// If `handle` is `Some` and points to a region that's big enough to contain `data`, then writes `data` into the region, and clears any remaining space at the end.
    /// `spare_capacity` is only applied when creating a fresh allocation; it is ignored when
    /// writing into an existing allocation that already fits the data.
    pub fn allocate_or_grow_and_write(&mut self, handle: &mut Option<Handle>, data: &[T], spare_capacity: u32, encoder: &mut wgpu::CommandEncoder) {
        if data.is_empty() {
            if let Some(h) = *handle {
                self.zero_handle(h, encoder);
            }
            return;
        }

        let needs_realloc = handle.map_or(true, |h| data.len() > h.size as usize);

        if needs_realloc {
            if let Some(h) = handle.take() {
                self.free_and_clear(h, encoder);
            }
            *handle = self.allocate(data.len() as u32 + spare_capacity, encoder);
        }

        if let Some(h) = *handle {
            self.write_handle(h, data, encoder);
        }
    }

    pub fn finish_belt(&mut self) {
        self.staging_belt.finish();
        self.dirty = false;
    }

    /// Returns true if the GPU buffer was reallocated since the last call, and resets the flag.
    /// Check this after `finish_belt` to know whether bind groups need to be recreated.
    pub fn take_reallocated(&mut self) -> bool {
        let r = self.reallocated;
        self.reallocated = false;
        r
    }

    /// Must be called after the encoder has been submitted.
    pub fn recall_belt(&mut self) {
        self.staging_belt.recall();
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

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.vec_len
    }
}
