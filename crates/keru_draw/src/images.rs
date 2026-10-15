use etagere::{Allocation, BucketedAtlasAllocator, size2};
use wgpu::*;
use std::collections::HashMap;

/// A loaded image stored in the atlas.
#[derive(Clone, Copy, Debug)]
pub struct LoadedImage {
    pub page: u16,
    pub alloc: Allocation,
    pub width: u32,
    pub height: u32,
    /// Internal ID for looking up SVG data (0 for raster images)
    pub(crate) id: u64,
}

struct AtlasPage {
    packer: BucketedAtlasAllocator,
}

// Number of mip levels generated for the atlas (levels 0..MIP_LEVELS, smallest = atlas_size >> (MIP_LEVELS-1)).
const MIP_LEVELS: u32 = 8;

const MIP_SHADER_WGSL: &str = r#"
@group(0) @binding(0) var src_tex: texture_2d_array<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d_array<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dst_size = textureDimensions(dst_tex);
    if (gid.x >= dst_size.x || gid.y >= dst_size.y) { return; }
    let layer = i32(gid.z);
    let sx = i32(gid.x) * 2;
    let sy = i32(gid.y) * 2;
    let c = (textureLoad(src_tex, vec2<i32>(sx,     sy    ), layer, 0) +
             textureLoad(src_tex, vec2<i32>(sx + 1, sy    ), layer, 0) +
             textureLoad(src_tex, vec2<i32>(sx,     sy + 1), layer, 0) +
             textureLoad(src_tex, vec2<i32>(sx + 1, sy + 1), layer, 0)) * 0.25;
    textureStore(dst_tex, vec2<u32>(gid.x, gid.y), layer, c);
}
"#;

/// Image renderer that manages texture atlases for SVG and raster images.
/// Images are rendered by using them as textures on shapes (e.g., white boxes).
pub struct ImageRenderer {
    device: Device,
    queue: Queue,

    atlas_size: u32,
    atlas_pages: Vec<AtlasPage>,

    pub(crate) texture_array: Texture,
    pub(crate) sampler: Sampler,

    mip_pipeline: wgpu::ComputePipeline,
    mip_bind_group_layout: wgpu::BindGroupLayout,

    needs_bind_group_rebuild: bool,
    needs_mipmaps: bool,

    /// Cache of raw SVG data for rerasterization
    svg_data_cache: HashMap<u64, Vec<u8>>,
    /// Counter for generating unique IDs
    next_id: u64,
}

impl ImageRenderer {
    /// Create a new ImageRenderer with default atlas size of 4096x4096
    pub fn new(device: &Device, queue: &Queue) -> Self {
        Self::new_with_atlas_size(device, queue, 4096)
    }

    /// Create a new ImageRenderer with custom atlas size
    pub fn new_with_atlas_size(device: &Device, queue: &Queue, atlas_size: u32) -> Self {
        // Create initial texture array with 1 layer
        let texture_array = create_texture_array(device, atlas_size, 1);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Image Atlas Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Linear,
            ..Default::default()
        });

        let (mip_pipeline, mip_bind_group_layout) = create_mip_pipeline(device);

        // Create initial atlas page
        let atlas_pages = vec![AtlasPage {
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
        }];

        Self {
            device: device.clone(),
            queue: queue.clone(),
            atlas_size,
            atlas_pages,
            texture_array,
            sampler,
            mip_pipeline,
            mip_bind_group_layout,
            needs_bind_group_rebuild: false,
            needs_mipmaps: false,
            svg_data_cache: HashMap::new(),
            next_id: 1,
        }
    }

    /// Load and rasterize an SVG, returning the loaded image.
    ///
    /// Returns None if the SVG couldn't be loaded/rasterized.
    pub fn load_svg(&mut self, svg_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        self.rasterize_and_store(svg_data, width, height)
    }

    /// Rerasterize an SVG at a new size if needed.
    ///
    /// Returns true if rerasterization occurred.
    pub fn rerasterize_svg_if_needed(&mut self, loaded: &mut LoadedImage, width: u32, height: u32) -> bool {
        if loaded.id == 0 {
            return false; // Not an SVG
        }

        if width == loaded.width && height == loaded.height {
            return false; // Already at desired size
        }

        // Get the cached SVG data
        if let Some(svg_data) = self.svg_data_cache.get(&loaded.id).cloned() {
            // Deallocate old texture space
            self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);

            // Rerasterize at new size
            if let Some(new_loaded) = self.rasterize_and_store_with_id(&svg_data, width, height, loaded.id) {
                *loaded = new_loaded;
                return true;
            }
        }

        false
    }

    /// Remove a loaded SVG and free its atlas space.
    pub fn unload_svg(&mut self, loaded: &LoadedImage) {
        self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);
        // Remove cached SVG data
        if loaded.id != 0 {
            self.svg_data_cache.remove(&loaded.id);
        }
    }

    /// Load a raster image from raw RGBA8 bytes, returning the loaded image.
    ///
    /// Returns None if the image couldn't be stored in the atlas.
    pub fn load_rgba8_image(&mut self, rgba_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        let mut owned = rgba_data.to_vec();
        premultiply_rgba8_in_place(&mut owned);
        self.store_image_data(&owned, width, height)
    }

    /// Load a raster image from encoded bytes (PNG, JPEG, etc.), returning the loaded image.
    ///
    /// Returns None if the image couldn't be decoded or stored.
    pub fn load_encoded_image(&mut self, image_data: &[u8]) -> Option<LoadedImage> {
        let img = image::load_from_memory(image_data).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut raw = rgba.into_raw();
        premultiply_rgba8_in_place(&mut raw);
        self.store_image_data(&raw, width, height)
    }

    /// Remove a loaded image and free its atlas space
    pub fn unload_image(&mut self, loaded: &LoadedImage) {
        self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);
    }

    /// Replace the pixels of a previously loaded raster image, reusing the same atlas slot when the dimensions are unchanged and reallocating otherwise. Returns the new [`LoadedImage`], or `None` if `rgba_data` doesn't match `width * height * 4` or is too big to store.
    pub fn replace_rgba8_image(&mut self, old: &LoadedImage, rgba_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        if rgba_data.len() != (width * height * 4) as usize {
            return None;
        }
        let mut owned = rgba_data.to_vec();
        premultiply_rgba8_in_place(&mut owned);
        self.replace_image(old, &owned, width, height)
    }

    /// Replace the pixels of a previously loaded raster image from encoded bytes. Returns the new [`LoadedImage`], or `None` if the bytes couldn't be decoded or stored.
    pub fn replace_encoded_image(&mut self, old: &LoadedImage, image_data: &[u8]) -> Option<LoadedImage> {
        let img = image::load_from_memory(image_data).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut raw = rgba.into_raw();
        premultiply_rgba8_in_place(&mut raw);
        self.replace_image(old, &raw, width, height)
    }

    fn replace_image(&mut self, old: &LoadedImage, premultiplied: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        if width == old.width && height == old.height {
            return Some(self.store_in_atlas(premultiplied, old.alloc, old.page as usize, width, height, old.id));
        }
        let new_loaded = self.store_image_data_with_id(premultiplied, width, height, old.id)?;
        self.atlas_pages[old.page as usize].packer.deallocate(old.alloc.id);
        Some(new_loaded)
    }

    // Since we use write_texture directly each time, this doesn't really fit the old "load_to_gpu" idea anymore.
    pub(crate) fn load_to_gpu(&mut self) -> bool {
        if self.needs_mipmaps {
            self.generate_mipmaps();
            self.needs_mipmaps = false;
        }

        let rebuilt = self.needs_bind_group_rebuild;
        self.needs_bind_group_rebuild = false;
        rebuilt
    }

    fn rasterize_and_store(&mut self, svg_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        // Generate unique ID and store SVG data
        let id = self.next_id;
        self.next_id += 1;
        self.svg_data_cache.insert(id, svg_data.to_vec());

        self.rasterize_and_store_with_id(svg_data, width, height, id)
    }

    fn rasterize_and_store_with_id(&mut self, svg_data: &[u8], width: u32, height: u32, id: u64) -> Option<LoadedImage> {
        // Parse SVG
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_data, &opt).ok()?;

        // Create pixmap for rasterization
        // we could keep a scratch buffer but it probably happens rarely enough that it's better to keep the memory. maybe
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

        // Calculate scale to fit SVG in requested dimensions
        let svg_size = tree.size();
        let scale_x = width as f32 / svg_size.width();
        let scale_y = height as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // Center the SVG when its aspect ratio differs from the requested size.
        let offset_x = (width as f32 - svg_size.width() * scale) * 0.5;
        let offset_y = (height as f32 - svg_size.height() * scale) * 0.5;
        let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

        // Render
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // the tiny_skia pixmap is already premultiplied
        let rgba_data = pixmap.take();

        self.store_image_data_with_id(&rgba_data, width, height, id)
    }

    fn store_image_data(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Option<LoadedImage> {
        self.store_image_data_with_id(rgba_data, width, height, 0)
    }

    fn store_image_data_with_id(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
        id: u64,
    ) -> Option<LoadedImage> {
        // Verify data size
        if rgba_data.len() != (width * height * 4) as usize {
            return None;
        }

        // Try to allocate in existing pages
        for page_idx in 0..self.atlas_pages.len() {
            if let Some(alloc) = self.atlas_pages[page_idx].packer.allocate(size2(width as i32, height as i32)) {
                return Some(self.store_in_atlas(rgba_data, alloc, page_idx, width, height, id));
            }
        }

        // Create new page
        let new_page_idx = self.make_new_page();
        if let Some(alloc) = self.atlas_pages[new_page_idx].packer.allocate(size2(width as i32, height as i32)) {
            return Some(self.store_in_atlas(rgba_data, alloc, new_page_idx, width, height, id));
        }

        // Image too large even for new page
        None
    }

    fn store_in_atlas(
        &mut self,
        rgba_data: &[u8],
        alloc: Allocation,
        page_idx: usize,
        width: u32,
        height: u32,
        id: u64,
    ) -> LoadedImage {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture_array,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: alloc.rectangle.min.x as u32,
                    y: alloc.rectangle.min.y as u32,
                    z: page_idx as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.needs_mipmaps = true;

        LoadedImage {
            page: page_idx as u16,
            alloc,
            width,
            height,
            id,
        }
    }

    fn make_new_page(&mut self) -> usize {
        let old_layers = self.atlas_pages.len() as u32;
        self.atlas_pages.push(AtlasPage {
            packer: BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32)),
        });
        self.grow_texture_array(old_layers);
        self.atlas_pages.len() - 1
    }

    fn grow_texture_array(&mut self, old_layers: u32) {
        let new_texture = create_texture_array(&self.device, self.atlas_size, self.atlas_pages.len() as u32);

        // Only mip 0 is copied, the other ones are regenerated.
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_texture(
            self.texture_array.as_image_copy(),
            new_texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.atlas_size,
                height: self.atlas_size,
                depth_or_array_layers: old_layers,
            },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        self.texture_array = new_texture;
        self.needs_bind_group_rebuild = true;
        self.needs_mipmaps = true;
    }

    fn generate_mipmaps(&self) {
        let num_layers = self.atlas_pages.len() as u32;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for mip in 1..MIP_LEVELS {
            let src_view = self.texture_array.create_view(&wgpu::TextureViewDescriptor {
                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: mip - 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(num_layers),
                ..Default::default()
            });
            let dst_view = self.texture_array.create_view(&wgpu::TextureViewDescriptor {
                format: Some(wgpu::TextureFormat::Rgba8Unorm),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(num_layers),
                ..Default::default()
            });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.mip_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&src_view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&dst_view) },
                ],
            });

            let mip_size = (self.atlas_size >> mip).max(1);
            let groups = (mip_size + 7) / 8;

            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.mip_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(groups, groups, num_layers);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn premultiply_rgba8_in_place(rgba_data: &mut [u8]) {
    for pixel in rgba_data.chunks_exact_mut(4) {
        let a = pixel[3] as u32;
        for c in &mut pixel[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
}

fn create_texture_array(device: &Device, size: u32, layers: u32) -> Texture {

    device.create_texture(&TextureDescriptor {
        label: Some("Image Atlas Texture Array"),
        size: Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
        mip_level_count: MIP_LEVELS,
        sample_count: 1,
        dimension: TextureDimension::D2,
        // Must be Rgba8Unorm: STORAGE_BINDING is not allowed on sRGB formats, and view_formats
        // can't include sRGB either when STORAGE_BINDING is set. Atlas data is linear; the
        // surface handles sRGB conversion.
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    })
}

fn create_mip_pipeline(device: &Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Mip Blit"),
        source: wgpu::ShaderSource::Wgsl(MIP_SHADER_WGSL.into()),
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Mip Blit"),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    (pipeline, bgl)
}
