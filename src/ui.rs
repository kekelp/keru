use crate::changes::{NodeWithDepth, PartialChanges};
use crate::interact::{HeldNodes, LastFrameClicks, MouseInputState, StoredClick};
use crate::math::*;
use crate::render::TypedGpuBuffer;
use crate::texture_atlas::*;
use crate::thread_local::thread_local_push;
use crate::*;
use copypasta::ClipboardContext;
use glyphon::Cache as GlyphonCache;
use glyphon::Viewport;

use rustc_hash::FxHashMap;
use slab::Slab;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferBindingType, ColorWrites, FilterMode,
    FragmentState, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState,
};

use std::{mem, time::Instant};

use bytemuck::{Pod, Zeroable};
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer};
use wgpu::{
    util::{self, DeviceExt},
    BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device, MultisampleState, Queue,
    RenderPipeline, SurfaceConfiguration, VertexBufferLayout, VertexStepMode,
};
use winit::{dpi::PhysicalPosition, keyboard::ModifiersState};

// todo: the sys split is no longer needed, lol.
pub struct Ui {
    pub(crate) nodes: Nodes,
    pub(crate) sys: System,
    pub(crate) format_scratch: String,
}

pub struct System {
    // todo: just put ROOT_I everywhere.
    pub root_i: usize,

    // in debug mode, draw invisible rects as well, for example V_STACKs.
    // usually these have filled = false (just the outline), but this is not enforced.
    pub debug_mode: bool,

    pub rects_generation: u32,
    pub debug_key_pressed: bool,

    pub mouse_status: MouseInputState,

    pub clipboard: ClipboardContext,

    pub key_mods: ModifiersState,

    pub gpu_rect_buffer: TypedGpuBuffer<RenderRect>,
    pub render_pipeline: RenderPipeline,

    pub base_uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub text: TextSystem,
    pub texture_atlas: TextureAtlas,

    pub rects: Vec<RenderRect>,
    // todo: keep a separate vec with the bounding boxes for faster mouse hit scans

    // stack for traversing
    pub traverse_stack: Vec<usize>,

    pub part: PartialBorrowStuff,

    pub clicked_stack: Vec<(Id, f32)>,
    pub mouse_hit_stack: Vec<(Id, f32)>,
    pub last_frame_clicks: LastFrameClicks,

    pub held_store: HeldNodes,
    pub dragged_store: HeldNodes,

    pub last_frame_click_released: Vec<StoredClick>,
    pub hovered: Vec<Id>,
    pub last_hovered: Id,

    pub focused: Option<Id>,

    pub size_scratch: Vec<f32>,
    pub(crate) relayouts_scrath: Vec<NodeWithDepth>,

    pub(crate) changes: PartialChanges,

    pub params_changed: bool,
    pub text_changed: bool,

    pub frame_t: f32,
    pub last_frame_timestamp: Instant,
}

pub struct PartialBorrowStuff {
    pub mouse_pos: PhysicalPosition<f32>,
    pub unifs: Uniforms,
    pub current_frame: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

impl Ui {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let gpu_rect_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("player bullet pos buffer"),
            // todo: I guess this should be growable
            contents: bytemuck::cast_slice(&[0.0; 2048]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let gpu_rect_buffer = TypedGpuBuffer::new(gpu_rect_buffer);
        let vert_buff_layout = VertexBufferLayout {
            array_stride: mem::size_of::<RenderRect>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &RenderRect::buffer_desc(),
        };

        let uniforms = Uniforms {
            size: Xy::new(config.width as f32, config.height as f32),
            t: 0.,
            _padding: 0.,
        };
        let resolution_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Resolution Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut texture_atlas = TextureAtlas::new(device);

        let _white_alloc = texture_atlas.allocate_image(include_bytes!("white.png"));

        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture sampler"),
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0f32,
            lod_max_clamp: 0f32,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
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
            ],
            label: Some("Fulgur Bind Group Layout"),
        });

        // Create the bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: resolution_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(texture_atlas.texture_view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("Fulgur Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shaders/box.wgsl").into()),
        });

        let primitive = PrimitiveState::default();

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vert_buff_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let glyphon_cache = GlyphonCache::new(&device);
        let glyphon_viewport = Viewport::new(&device, &glyphon_cache);

        let mut atlas = TextAtlas::new(device, queue, &glyphon_cache, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let text_areas = Vec::with_capacity(50);

        let mut node_hashmap = FxHashMap::with_capacity_and_hasher(100, Default::default());

        let mut nodes = Slab::with_capacity(100);
        let root_i = nodes.insert(NODE_ROOT);
        let root_map_entry = NodeMapEntry {
            last_parent: usize::default(),
            last_frame_touched: u64::MAX,
            slab_i: root_i,
            n_twins: 0,
        };

        let root_parent = Parent::new(root_i, EMPTY_HASH);
        thread_local_push(&root_parent);

        node_hashmap.insert(NODE_ROOT_ID, root_map_entry);

        let nodes = Nodes {
            node_hashmap,
            nodes,
        };

        Self {
            nodes,
            format_scratch: String::with_capacity(1024),

            sys: System {
                root_i,
                debug_mode: false,
                debug_key_pressed: false,

                mouse_status: MouseInputState::default(),

                clipboard: ClipboardContext::new().unwrap(),
                key_mods: ModifiersState::default(),

                text: TextSystem {
                    cache,
                    atlas,
                    text_renderer,
                    font_system,
                    text_areas,
                    glyphon_cache,
                    glyphon_viewport,
                },

                texture_atlas,

                render_pipeline,
                rects: Vec::with_capacity(20),

                gpu_rect_buffer,
                base_uniform_buffer: resolution_buffer,
                bind_group,

                traverse_stack: Vec::with_capacity(50),

                size_scratch: Vec::with_capacity(15),
                relayouts_scrath: Vec::with_capacity(15),

                part: PartialBorrowStuff {
                    mouse_pos: PhysicalPosition { x: 0., y: 0. },
                    current_frame: FIRST_FRAME,
                    unifs: uniforms,
                },

                clicked_stack: Vec::with_capacity(50),
                mouse_hit_stack: Vec::with_capacity(50),
                last_frame_clicks: LastFrameClicks::new(),

                held_store: HeldNodes::default(),
                dragged_store: HeldNodes::default(),

                last_frame_click_released: Vec::with_capacity(5),

                hovered: Vec::with_capacity(15),
                last_hovered: Id(0),
                focused: None,

                frame_t: 0.0,

                params_changed: true,
                text_changed: true,

                last_frame_timestamp: Instant::now(),
                rects_generation: 1,

                changes: PartialChanges::new(),
            },
        }
    }
}
