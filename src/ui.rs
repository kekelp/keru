use crate::changes::{NodeWithDepth, PartialChanges};
use crate::math::*;
use crate::render::TypedGpuBuffer;
use crate::texture_atlas::*;
use crate::*;
use crate::node::*;
use crate::interact::*;
use crate::render_rect::*;

use crate::math::Axis::*;

use basic_window_loop::basic_depth_stencil_state;
use copypasta::ClipboardContext;
use glyphon::Cache as GlyphonCache;
use glyphon::Viewport;

use interact::PendingMousePress;
use node::Node;
use rustc_hash::FxHashMap;
use slab::Slab;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferBindingType, ColorWrites, FilterMode,
    FragmentState, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState,
};

use std::ops::{Index, IndexMut};
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

pub(crate) struct System {
    // todo: just put ROOT_I everywhere.
    pub root_i: usize,

    // in debug mode, draw invisible rects as well, for example V_STACKs.
    // usually these have filled = false (just the outline), but this is not enforced.
    pub debug_mode: bool,

    pub debug_key_pressed: bool,

    pub _clipboard: ClipboardContext,

    pub key_mods: ModifiersState,

    pub gpu_rect_buffer: TypedGpuBuffer<RenderRect>,
    pub render_pipeline: RenderPipeline,

    pub base_uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub text: TextSystem,
    pub texture_atlas: TextureAtlas,

    pub z_cursor: f32,
    pub rects: Vec<RenderRect>,
    pub invisible_but_clickable_rects: Vec<RenderRect>,
    // todo: keep a separate vec with the bounding boxes for faster mouse hit scans

    pub part: PartialBorrowStuff,

    pub mouse_hit_stack: Vec<(Id, f32)>,

    pub unresolved_click_presses: Vec<PendingMousePress>,
    pub last_frame_mouse_events: Vec<MouseEvent>,


    pub hovered: Vec<Id>,

    pub focused: Option<Id>,

    pub size_scratch: Vec<f32>,
    pub relayouts_scrath: Vec<NodeWithDepth>,
    // this is used exclusively for debug messages
    pub partial_relayout_count: u32,

    pub changes: PartialChanges,

    pub frame_t: f32,
    pub last_frame_timestamp: Instant,
}

pub(crate) struct PartialBorrowStuff {
    pub mouse_pos: PhysicalPosition<f32>,
    pub unifs: Uniforms,
    pub current_frame: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub(crate) struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

impl Ui {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let gpu_rect_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("player bullet pos buffer"),
            // todo: I guess this should be growable
            contents: {
                let warning = "todo: make this growable";
                bytemuck::cast_slice(&[0.0; 2048])
            },
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

        let _white_alloc = texture_atlas.allocate_image(include_bytes!("textures/white.png"));
        // let _debug_alloc = texture_atlas.allocate_image(include_bytes!("textures/debug.png"));

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
                entry_point: Some("vs_main"),
                buffers: &[vert_buff_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive,
            depth_stencil: Some(basic_depth_stencil_state()),
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
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), Some(basic_depth_stencil_state()));

        let text_areas = Vec::with_capacity(50);

        let mut node_hashmap = FxHashMap::with_capacity_and_hasher(100, Default::default());

        let mut nodes = Slab::with_capacity(100);
        let root_i = nodes.insert(NODE_ROOT);
        let root_map_entry = NodeMapEntry {
            last_frame_touched: u64::MAX,
            slab_i: root_i,
            n_twins: 0,
        };

        node_hashmap.insert(NODE_ROOT_ID, root_map_entry);

        let nodes = Nodes {
            node_hashmap,
            nodes,
        };

        Self {
            nodes,
            format_scratch: String::with_capacity(1024),

            sys: System {
                z_cursor: 0.0,
                root_i,
                debug_mode: false,
                debug_key_pressed: false,

                _clipboard: ClipboardContext::new().unwrap(),
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
                rects: Vec::with_capacity(50),
                invisible_but_clickable_rects: Vec::with_capacity(20),

                gpu_rect_buffer,
                base_uniform_buffer: resolution_buffer,
                bind_group,

                size_scratch: Vec::with_capacity(15),
                relayouts_scrath: Vec::with_capacity(15),
                partial_relayout_count: 0,

                part: PartialBorrowStuff {
                    mouse_pos: PhysicalPosition { x: 0., y: 0. },
                    current_frame: FIRST_FRAME,
                    unifs: uniforms,
                },

                mouse_hit_stack: Vec::with_capacity(50),

                unresolved_click_presses: Vec::with_capacity(20),
                last_frame_mouse_events: Vec::with_capacity(20),

                hovered: Vec::with_capacity(15),
                focused: None,

                frame_t: 0.0,

                last_frame_timestamp: Instant::now(),

                changes: PartialChanges::new(),
            },
        }
    }

    pub fn key_mods(&self) -> &ModifiersState {
        return &self.sys.key_mods;
    }

    pub fn base_uniform_buffer(&self) -> &Buffer {
        return &self.sys.base_uniform_buffer;
    }
}



#[derive(Debug, Clone, Copy)]
pub struct NodeMapEntry {
    pub last_frame_touched: u64,

    // keeping track of the twin situation.
    // This is the number of twins of a node that showed up SO FAR in the current frame. it gets reset every frame (on refresh().)
    // for the 0-th twin of a family, this will be the total number of clones of itself around. (not including itself, so starts at zero).
    // the actual twins ARE twins, but they don't HAVE twins, so this is zero.
    // for this reason, "clones" or "copies" would be better names, but those words are loaded in rust
    // reproduction? replica? imitation? duplicate? version? dupe? replication? mock? carbon?
    pub n_twins: u32,
    pub slab_i: usize,
}
impl NodeMapEntry {
    pub fn new(frame: u64, new_i: usize) -> Self {
        return Self {
            last_frame_touched: frame,
            n_twins: 0,
            slab_i: new_i,
        };
    }

    pub fn refresh(&mut self, frame: u64) -> usize {
        self.last_frame_touched = frame;
        self.n_twins = 0;
        return self.slab_i;
    }
}

#[derive(Debug)]
pub struct Nodes {
    // todo: make faster or something
    pub node_hashmap: FxHashMap<Id, NodeMapEntry>,
    pub nodes: Slab<Node>,
}
impl Nodes {
    pub fn get_by_id(&mut self, id: &Id) -> Option<&mut Node> {
        let i = self.node_hashmap.get(id)?.slab_i;
        return self.nodes.get_mut(i);
    }
}
impl Index<usize> for Nodes {
    type Output = Node;
    fn index(&self, i: usize) -> &Self::Output {
        return &self.nodes[i];
    }
}
impl IndexMut<usize> for Nodes {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        return &mut self.nodes[i];
    }
}

impl PartialBorrowStuff {
    pub fn mouse_hit_rect(&self, rect: &RenderRect) -> bool {
        // rects are rebuilt whenever they change, they don't have to be skipped based on a timestamp or anything like that.
        // in the future if we do a click detection specific datastructure it might use a timestamp, maybe? probably not.

        let mut mouse_pos = (
            self.mouse_pos.x / self.unifs.size[X],
            1.0 - (self.mouse_pos.y / self.unifs.size[Y]),
        );

        // transform mouse_pos into "opengl" centered coordinates
        mouse_pos.0 = (mouse_pos.0 * 2.0) - 1.0;
        mouse_pos.1 = (mouse_pos.1 * 2.0) - 1.0;

        let aabb_hit = rect.rect[X][0] < mouse_pos.0
            && mouse_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < mouse_pos.1
            && mouse_pos.1 < rect.rect[Y][1];

        if !aabb_hit {
            return false;
        }

        match rect.read_shape() {
            Shape::Rectangle { corner_radius: _ } => {
                return aabb_hit;
            }
            Shape::Circle => {
                // Calculate the circle center and radius
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;
    
                // Check if the mouse is within the circle
                let dx = mouse_pos.0 - center_x;
                let dy = mouse_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Ring { width } => {
                // scale to correct coordinates
                // width should have been a Len anyway so this will have to change
                let width = width / self.unifs.size[X];

                let aspect = self.unifs.size[X] / self.unifs.size[Y];
                 // Calculate the ring's center and radii
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let outer_radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;
                let inner_radius = outer_radius - width;
    
                // Check if the mouse is within the ring
                let dx = mouse_pos.0 - center_x;
                let dy = (mouse_pos.1 - center_y) / aspect;
                let distance_squared = dx * dx + dy * dy;
                return distance_squared <= outer_radius * outer_radius
                    && distance_squared >= inner_radius * inner_radius;

                // in case there's any doubts, this was awful, it would be a lot better to have the click specific datastruct so that everything there can be in pixels
            }
        }
    }
}

impl Ui {
    pub fn debug_mode(&self) -> bool {
        return self.sys.debug_mode;
    }
}