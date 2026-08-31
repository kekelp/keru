use etagere::{Allocation, BucketedAtlasAllocator, size2};
use image::RgbaImage;
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
    image: RgbaImage,
    dirty: bool,
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
    atlas_size: u32,
    atlas_pages: Vec<AtlasPage>,

    pub(crate) texture_array: Texture,
    pub(crate) sampler: Sampler,

    mip_pipeline: wgpu::ComputePipeline,
    mip_bind_group_layout: wgpu::BindGroupLayout,

    needs_texture_array_rebuild: bool,

    /// Cache of raw SVG data for rerasterization
    svg_data_cache: HashMap<u64, Vec<u8>>,
    /// Counter for generating unique IDs
    next_id: u64,
}

impl ImageRenderer {
    /// Create a new ImageRenderer with default atlas size of 4096x4096
    pub fn new(device: &Device, _queue: &Queue) -> Self {
        Self::new_with_atlas_size(device, _queue, 4096)
    }

    /// Create a new ImageRenderer with custom atlas size
    pub fn new_with_atlas_size(device: &Device, _queue: &Queue, atlas_size: u32) -> Self {
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
            image: RgbaImage::new(atlas_size, atlas_size),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            dirty: false,
        }];

        Self {
            atlas_size,
            atlas_pages,
            texture_array,
            sampler,
            mip_pipeline,
            mip_bind_group_layout,
            needs_texture_array_rebuild: false,
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
        self.store_image_data(rgba_data, width, height)
    }

    /// Load a raster image from encoded bytes (PNG, JPEG, etc.), returning the loaded image.
    ///
    /// Returns None if the image couldn't be decoded or stored.
    pub fn load_encoded_image(&mut self, image_data: &[u8]) -> Option<LoadedImage> {
        let img = image::load_from_memory(image_data).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        self.load_rgba8_image(rgba.as_raw(), width, height)
    }

    /// Remove a loaded image and free its atlas space
    pub fn unload_image(&mut self, loaded: &LoadedImage) {
        self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);
    }

    /// Upload textures to GPU. Returns true if the texture array was rebuilt.
    pub(crate) fn load_to_gpu(&mut self, device: &Device, queue: &Queue) -> bool {
        if !self.needs_texture_array_rebuild && !self.atlas_pages.iter().any(|p| p.dirty) {
            return false;
        }

        let rebuilt = if self.needs_texture_array_rebuild {
            self.rebuild_texture_array(device, queue);
            self.needs_texture_array_rebuild = false;
            true
        } else {
            self.update_texture_array(queue);
            false
        };

        self.generate_mipmaps(device, queue);

        rebuilt
    }

    // Internal methods

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

        let transform = tiny_skia::Transform::from_scale(scale, scale);

        // Render
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // unpremultiply alpha
        let mut rgba_data = pixmap.take();
        for pixel in rgba_data.chunks_exact_mut(4) {
            let a = pixel[3];
            if a > 0 && a < 255 {
                let a_f = a as f32 / 255.0;
                pixel[0] = (pixel[0] as f32 / a_f).round() as u8;
                pixel[1] = (pixel[1] as f32 / a_f).round() as u8;
                pixel[2] = (pixel[2] as f32 / a_f).round() as u8;
            }
        }

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
        // Copy image data to atlas
        let dst_x = alloc.rectangle.min.x as u32;
        let dst_y = alloc.rectangle.min.y as u32;

        for y in 0..height {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                let pixel = image::Rgba([
                    rgba_data[offset],
                    rgba_data[offset + 1],
                    rgba_data[offset + 2],
                    rgba_data[offset + 3],
                ]);
                self.atlas_pages[page_idx].image.put_pixel(dst_x + x, dst_y + y, pixel);
            }
        }

        self.atlas_pages[page_idx].dirty = true;

        LoadedImage {
            page: page_idx as u16,
            alloc,
            width,
            height,
            id,
        }
    }

    fn make_new_page(&mut self) -> usize {
        self.atlas_pages.push(AtlasPage {
            image: RgbaImage::new(self.atlas_size, self.atlas_size),
            packer: BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32)),
            dirty: true,
        });
        self.needs_texture_array_rebuild = true;
        self.atlas_pages.len() - 1
    }

    fn rebuild_texture_array(&mut self, device: &Device, queue: &Queue) {
        let num_pages = self.atlas_pages.len().max(1);

        self.texture_array = create_texture_array(device, self.atlas_size, num_pages as u32);

        // Upload all pages
        for (page_idx, page) in self.atlas_pages.iter_mut().enumerate() {
            upload_texture_page(queue, &self.texture_array, &page.image, page_idx as u32, self.atlas_size);
            page.dirty = false;
        }
    }

    fn update_texture_array(&mut self, queue: &Queue) {
        for (page_idx, page) in self.atlas_pages.iter_mut().enumerate() {
            if page.dirty {
                upload_texture_page(queue, &self.texture_array, &page.image, page_idx as u32, self.atlas_size);
                page.dirty = false;
            }
        }
    }

    fn generate_mipmaps(&self, device: &Device, queue: &Queue) {
        let num_layers = self.atlas_pages.len() as u32;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        queue.submit(std::iter::once(encoder.finish()));
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
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
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

fn upload_texture_page(queue: &Queue, texture: &Texture, image: &RgbaImage, layer: u32, size: u32) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { z: layer, ..Default::default() },
            aspect: wgpu::TextureAspect::All,
        },
        image.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
    );
}
