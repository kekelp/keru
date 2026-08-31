use crate::*;

fn bind_group_layout_entries() -> [BindGroupLayoutEntry; 7] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            // This is visible in vertex to get the size. Could be avoided, but I don't know if it's a big deal.
            visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2Array,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                multisampled: false,
                view_dimension: TextureViewDimension::D2Array,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        },
        GpuHeap::<GlyphQuad>::bind_group_layout_entry(3),
        // Params uniform buffer
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(mem::size_of::<Params>() as u64),
            },
            count: None,
        },
        // Box data storage buffer
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Group transforms storage buffer
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    ]
}

/// Configuration parameters for the text renderer.
pub struct TextRendererParams {
    /// Size of texture atlas pages used for glyph caching.
    pub atlas_page_size: AtlasPageSize,
}
impl Default for TextRendererParams {
    fn default() -> Self {
        // 2048 is guaranteed to work everywhere that webgpu supports, and it seems both small enough that it's fine to allocate it upfront even if a smaller one would have been fine, and big enough that even on gpus that could hold 8k textures, I don't feel too bad about using multiple 2k pages instead of a single big 8k one
        // Ideally you'd still with small pages and grow them until the max texture dim, but having cache eviction, multiple pages, AND page growing seems a bit too much for now
        let atlas_page_size = AtlasPageSize::DownlevelWrbgl2Max; // 2048
        Self {
            atlas_page_size,
        }
    }
}
/// Determines the size of texture atlas pages for glyph storage.
pub enum AtlasPageSize {
    /// Fixed size in pixels.
    Flat(u32),
    /// Use the current device's maximum texture size.
    CurrentDeviceMax,
    /// Use WebGL2 downlevel maximum (2048px).
    DownlevelWrbgl2Max,
    /// Use general downlevel maximum (2048px).
    DownlevelMax,
    /// Use WGPU's default maximum (8192px).
    WgpuMax,
}
impl AtlasPageSize {
    /// Get the size in pixels for this atlas page size option.
    pub fn size(self, device: &Device) -> u32 {
        match self {
            AtlasPageSize::Flat(i) => i,
            AtlasPageSize::DownlevelWrbgl2Max => Limits::downlevel_defaults().max_texture_dimension_2d,
            AtlasPageSize::DownlevelMax => Limits::downlevel_webgl2_defaults().max_texture_dimension_2d,
            AtlasPageSize::WgpuMax => Limits::default().max_texture_dimension_2d,
            AtlasPageSize::CurrentDeviceMax => device.limits().max_texture_dimension_2d,
        }
    }
}

impl TextRenderer {
    /// Create a new TextRenderer with the specified parameters.
    pub(crate) fn new_with_params(
        device: Device,
        queue: Queue,
        format: TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        atlas_size: u32,
        box_data: &GpuSlab<BoxGpu>,
        group_transforms: &GpuSlab<GroupTransform>,
        glyph_quads: &GpuHeap<GlyphQuad>,
    ) -> Self {
        let srgb = format.is_srgb();

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sampler"),
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            lod_min_clamp: 0f32,
            lod_max_clamp: 0f32,
            ..Default::default()
        });

        let vertex_spirv = include_bytes!("../slangc_output/text.vert.spv");
        let fragment_spirv = include_bytes!("../slangc_output/text.frag.spv");

        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("keru_text vertex shader"),
            source: wgpu::util::make_spirv(vertex_spirv),
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("keru_text fragment shader"),
            source: wgpu::util::make_spirv(fragment_spirv),
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphQuad>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Uint32,      // pos_packed
                1 => Uint32,      // dim_packed
                2 => Uint32,      // uv_origin_packed
                3 => Uint32,      // color
                4 => Uint32,      // flags_and_page
                5 => Uint32,      // box_index
                6 => Uint32,    // _padding1
                7 => Uint32,    // _padding2
            ],
        };

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("params"),
            size: mem::size_of::<Params>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entries = bind_group_layout_entries();
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("bind group layout"),
        });

        // Create initial empty atlas pages for initial bind group creation
        let mut mask_atlas_pages = vec![AtlasPage {
            image: GrayImage::from_pixel(atlas_size, atlas_size, Luma([0])),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            needs_upload: true,
        }];

        let mut color_atlas_pages = vec![AtlasPage {
            image: RgbaImage::from_pixel(atlas_size, atlas_size, Rgba([0, 0, 0, 0])),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            needs_upload: true,
        }];

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("keru_text pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: Some("main"),
                buffers: &[Some(vertex_buffer_layout)],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: Some("main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::default(),
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let (mask_texture_array, color_texture_array) = rebuild_texture_arrays(
            &device,
            &queue,
            atlas_size,
            &mut mask_atlas_pages,
            &mut color_atlas_pages,
            srgb,
        );

        let bind_group = create_bind_group(
            &device,
            &mask_texture_array,
            &color_texture_array,
            glyph_quads,
            &sampler,
            &params_buffer,
            box_data,
            group_transforms,
            &bind_group_layout,
        );

        return Self {
            device,
            queue,
            atlas_size,
            mask_texture_array,
            color_texture_array,
            bind_group,
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            srgb,
        };
    }
}


pub(crate) fn create_bind_group(
    device: &wgpu::Device,
    mask_texture_array: &wgpu::Texture,
    color_texture_array: &wgpu::Texture,
    glyph_quads: &GpuHeap<GlyphQuad>,
    sampler: &wgpu::Sampler,
    params_buffer: &wgpu::Buffer,
    box_data: &GpuSlab<BoxGpu>,
    group_transforms: &GpuSlab<GroupTransform>,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {

    let mask_view = mask_texture_array.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    let color_view = color_texture_array.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    let entries = [
        wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&mask_view),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(&color_view),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(sampler),
        },
        glyph_quads.bind_group_entry(3),
        wgpu::BindGroupEntry {
            binding: 4,
            resource: params_buffer.as_entire_binding(),
        },
        box_data.bind_group_entry(5),
        group_transforms.bind_group_entry(6),
    ];

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &entries,
        label: Some("bind group"),
    })
}

pub(crate) fn rebuild_texture_arrays(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas_size: u32,
    mask_atlas_pages: &mut [AtlasPage<GrayImage>],
    color_atlas_pages: &mut [AtlasPage<RgbaImage>],
    surface_is_srgb: bool,
) -> (wgpu::Texture, wgpu::Texture) {
    // Create mask texture array
    let mask_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mask atlas array"),
        size: wgpu::Extent3d {
            width: atlas_size,
            height: atlas_size,
            depth_or_array_layers: mask_atlas_pages.len() as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for (i, page) in mask_atlas_pages.iter_mut().enumerate() {
        if page.needs_upload {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &mask_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                &page.image.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(page.image.width()),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: page.image.width(),
                    height: page.image.height(),
                    depth_or_array_layers: 1,
                },
            );
            page.needs_upload = false;
        }
    }

    let color_texture_format = if surface_is_srgb {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    };

    // Create color texture array
    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color atlas array"),
        size: wgpu::Extent3d {
            width: atlas_size,
            height: atlas_size,
            depth_or_array_layers: color_atlas_pages.len() as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: color_texture_format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for (i, page) in color_atlas_pages.iter_mut().enumerate() {
        if page.needs_upload {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &color_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                &page.image.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(page.image.width() * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: page.image.width(),
                    height: page.image.height(),
                    depth_or_array_layers: 1,
                },
            );
            page.needs_upload = false;
        }
    }

    (mask_tex, color_tex)
}
