use crate::*;

use crate::math::Axis::*;

use basic_window_loop::basic_depth_stencil_state;
// use copypasta::ClipboardContext;
use glam::DVec2;
use glyphon::Cache as GlyphonCache;
use glyphon::Viewport;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferBindingType, ColorWrites, FilterMode,
    FragmentState, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState,
};
use winit::dpi::PhysicalSize;
use winit_key_events::KeyInput;
use winit_mouse_events::MouseInput;

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::time::Duration;
use std::{mem, time::Instant};

use bytemuck::{Pod, Zeroable};
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer};
use wgpu::{
    util::{self, DeviceExt},
    BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device, MultisampleState, Queue,
    RenderPipeline, SurfaceConfiguration, VertexBufferLayout, VertexStepMode,
};
use winit::keyboard::ModifiersState;

pub(crate) static T0: LazyLock<Instant> = LazyLock::new(Instant::now);

pub(crate) fn ui_time_f32() -> f32 {
    return T0.elapsed().as_secs_f32();
}

/// The central struct of the library, representing the whole GUI state.
/// 
/// To create a new [`Ui`] instance, use [`Ui::new`].
/// 
/// To build a GUI, add nodes to the [`Ui`] by calling [`Ui::add`].
/// 
/// To react to mouse clicks and other node events, call [`Ui::is_clicked`] and similar methods.
/// 
/// To integrate [`Ui`] with your `winit` event loop, pass all your `winit` events to [`Ui::window_event`].
/// 
/// To render the GUI to the screen, call [`Ui::render`]. 
pub struct Ui {
    pub(crate) nodes: Nodes,
    pub(crate) sys: System,
    pub(crate) format_scratch: String,
}

static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct System {
    // in inspect mode, draw invisible rects as well, for example V_STACKs.
    // usually these have filled = false (just the outline), but this is not enforced.
    pub inspect_mode: bool,

    pub unique_id: u64,
    pub theme: Theme,
    pub debug_key_pressed: bool,

    pub new_ui_input: u8,
    pub new_external_events: bool,

    // pub clipboard: ClipboardContext,

    // todo: remove and put in mouse_input
    pub key_mods: ModifiersState,

    pub gpu_rect_buffer: TypedGpuBuffer<RenderRect>,
    pub render_pipeline: RenderPipeline,

    pub base_uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub text: TextSystem,
    pub texture_atlas: TextureAtlas,

    pub z_cursor: f32,
    pub rects: Vec<RenderRect>,

    pub click_rects: Vec<ClickRect>,

    pub scroll_rects: Vec<ClickRect>,

    pub unifs: Uniforms,
    pub current_frame: u64,
    pub last_frame_end_fake_time: u64,
    pub second_last_frame_end_fake_time: u64,
    pub third_last_frame_end_fake_time: u64,

    pub mouse_hit_stack: Vec<(Id, f32)>,

    // mouse input needs to be Id based, not NodeI based, because you can hold a button for several frames
    pub mouse_input: MouseInput<Id>,
    pub key_input: KeyInput,

    #[cfg(debug_assertions)]
    pub inspect_hovered: Option<Id>,

    pub hovered: Vec<Id>,
    pub hovered_scroll_area: Option<Id>,

    pub focused: Option<Id>,

    // this is used exclusively for info messages
    pub partial_relayout_count: u32,

    pub old_child_collect: Vec<NodeI>,
    pub new_child_collect: Vec<NodeI>,
    pub added_nodes: Vec<NodeI>,
    // nodes that were removed and were direct children of still-visible nodes. Among other things, this means that them disappearing has to trigger a partial relayout.
    pub direct_removed_nodes: Vec<NodeI>,
    // nodes that were removed "automatically" as a consequence of their parent or grandparent being directly removed. Aka orphaned nodes. These ones don't cause relayouts.
    pub indirect_removed_nodes: Vec<NodeI>,

    pub changes: PartialChanges,

    // move to changes oalgo
    pub anim_render_timer: AnimationRenderTimer,
}

pub(crate) struct AnimationRenderTimer(Option<Instant>);

impl AnimationRenderTimer {
    fn default() -> Self {
        Self(None)
    }

    pub(crate) fn push_new(&mut self, duration: Duration) {
        let now = Instant::now();
        let new_end = now + duration;

        if let Some(end) = self.0 {
            if new_end > end {
                *self = AnimationRenderTimer(Some(new_end));
            }
        } else {
            *self = AnimationRenderTimer(Some(new_end));
        }
    }

    pub(crate) fn is_live(&mut self) -> bool {
        if let Some(end) = self.0 {
            let is_live = Instant::now() < end;
            if !is_live {
                *self = AnimationRenderTimer(None);
            }
            return is_live;
        }
        false
    }
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
        // initialize the static T0
        LazyLock::force(&T0);
        
        let gpu_rect_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Keru rectangle buffer"),
            // todo: I guess this should be growable
            contents: {
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
            label: Some("Keru Bind Group Layout"),
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
            label: Some("Keru Bind Group"),
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

        let nodes = Nodes::new();

        let third_last_frame_end_fake_time = 3;
        let second_last_frame_end_fake_time = 4;
        let last_frame_end_fake_time = 5;

        Self {
            nodes,
            format_scratch: String::with_capacity(1024),

            sys: System {
                unique_id: INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed),
                z_cursor: 0.0,
                theme: KERU_DARK,
                inspect_mode: false,
                debug_key_pressed: false,

                new_ui_input: 2,
                new_external_events: true,

                // clipboard: ClipboardContext::new().unwrap(),
                key_mods: ModifiersState::default(),

                text: TextSystem {
                    cache,
                    atlas,
                    text_renderer,
                    font_system,
                    text_areas,
                    glyphon_viewport,
                },

                texture_atlas,

                render_pipeline,
                rects: Vec::with_capacity(100),
                
                click_rects: Vec::with_capacity(50),
                scroll_rects: Vec::with_capacity(20),

                gpu_rect_buffer,
                base_uniform_buffer: resolution_buffer,
                bind_group,

                partial_relayout_count: 0,

                current_frame: FIRST_FRAME,
                third_last_frame_end_fake_time,
                second_last_frame_end_fake_time,
                last_frame_end_fake_time,

                unifs: uniforms,

                mouse_hit_stack: Vec::with_capacity(50),

                mouse_input: MouseInput::default(),
                key_input: KeyInput::default(),

                // todo: maybe remove and use mouse_input.current_tag()? There was never a point in having multiple hovereds
                hovered: Vec::with_capacity(15),
                hovered_scroll_area: None,

                #[cfg(debug_assertions)]
                inspect_hovered: None,
            
                old_child_collect: Vec::with_capacity(10),
                new_child_collect: Vec::with_capacity(10),
                added_nodes: Vec::with_capacity(30),
                direct_removed_nodes: Vec::with_capacity(30),
                indirect_removed_nodes: Vec::with_capacity(30),

                focused: None,

                anim_render_timer: AnimationRenderTimer::default(),

                changes: PartialChanges::new(),
            },
        }
    }

    /// Returns a reference the `winit::ModifiersState` instance that the `Ui` stores and updates.
    /// 
    /// At the cost of some coupling, this can be reused in other parts of the program.
    pub fn key_mods(&self) -> &ModifiersState {
        return &self.sys.key_mods;
    }

    /// Returns a reference to a GPU buffer holding basic information.
    /// 
    /// At the cost of some coupling, this can be reused in other rendering jobs.
    /// 
    /// Example usage in shader:
    /// ```wgpu
    /// struct Uniforms {
    ///     @location(1) screen_resolution: vec2f,
    ///     @location(0) t: f32,
    /// };
    /// ```
    pub fn base_uniform_buffer(&self) -> &Buffer {
        return &self.sys.base_uniform_buffer;
    }

    /// Set inspect mode. When inspect mode is active, all nodes will be shown, including stacks and containers. 
    pub fn set_inspect_mode(&mut self, inspect_mode: bool) {
        if self.inspect_mode() != inspect_mode {
            self.sys.changes.tree_changed = true;
        }
        self.sys.inspect_mode = inspect_mode;
    }

    /// Get the current inspect mode state.
    /// When inspect mode is active, all nodes will be shown, including stacks and containers.
    pub fn inspect_mode(&self) -> bool {
        return self.sys.inspect_mode;
    }

    /// Get a reference to the active theme.
    pub fn theme(&mut self) -> &mut Theme {
        return &mut self.sys.theme;
    }

    pub fn current_frame(&self) -> u64 {
        return self.sys.current_frame;
    }

    pub fn unique_id(&self) -> u64 {
        return self.sys.unique_id;
    }

    pub fn push_external_event(&mut self) {
        self.sys.new_external_events = true;
    }

    /// Returns `true` if the [`Ui`] needs to be updated.
    /// 
    /// This is true when the [`Ui`] received an input that it cares about, such as a click on a clickable element, or when the user explicitly notified it with [`Ui::push_external_event()`].
    ///  
    /// In a typical `winit` loop for an application that only updates in response to user input, this function is what decides if the [`Ui`] building code should be rerun.
    /// 
    /// In applications that update on every frame regardless of user input, like games or simulations, the [`Ui`] building code should be rerun on every frame unconditionally, so this function isn't useful.
    pub fn needs_update(&mut self) -> bool {
        return self.sys.new_ui_input > 0 ||
            self.sys.new_external_events;
    }

    /// Returns `true` if the [`Ui`] needs to be updated or rerendered.
    /// 
    /// In a typical `winit` loop for an application that only updates in response to user input, this function is what decides if `winit::Window::request_redraw()` should be called.
    /// 
    /// An application that works like this can also wake up in response to external events, but it has to be explicitely told to, with [`Ui::push_external_event()`].
    /// 
    /// For an application that updates on every frame regardless of user input, like a game or a simulation, `request_redraw()` should be called on every frame unconditionally, so this function isn't useful.
    /// 
    pub fn event_loop_needs_to_wake(&mut self) -> bool {
        return self.needs_update() || self.needs_rerender();
    }

    pub fn cursor_position(&self) -> DVec2 {
        return self.sys.mouse_input.cursor_position();
    }

    // todo: expose functions directly instead of the inner struct
    pub fn key_input(&self) -> &KeyInput {
        return &self.sys.key_input;
    }

    pub(crate) fn set_new_ui_input(&mut self) {
        // Anti state-tearing: always update two times
        // Or rather, anti get-stuck-in-a-state-teared-frame. The state tearing is still there for one frame.
        self.sys.new_ui_input = 2;
    }

    /// Resize the `Ui`. 
    /// Updates the `Ui`'s internal state, and schedules a full relayout to adapt to the new size.
    /// Called by [`Ui::window_event`].
    pub(crate) fn resize(&mut self, size: &PhysicalSize<u32>) {        
        self.sys.changes.full_relayout = true;
        
        self.sys.unifs.size[X] = size.width as f32;
        self.sys.unifs.size[Y] = size.height as f32;

        self.sys.changes.resize = true;
        self.set_new_ui_input();
    }
}

impl Ui {
    pub(crate) fn hit_click_rect(&self, rect: &ClickRect) -> bool {
        let size = self.sys.unifs.size;
        let cursor_pos = (
            self.cursor_position().x as f32 / size[X],
            self.cursor_position().y as f32 / size[Y],
        );

        let aabb_hit = rect.rect[X][0] < cursor_pos.0
            && cursor_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < cursor_pos.1
            && cursor_pos.1 < rect.rect[Y][1];

        if aabb_hit == false {
            return false;
        }

        let node_i = rect.i;


        match self.nodes[node_i].params.rect.shape {
            Shape::Rectangle { corner_radius: _ } => {
                return true;
            }
            Shape::Circle => {
                // Calculate the circle center and radius
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                // Check if the mouse is within the circle
                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Ring { width } => {
                // scale to correct coordinates
                // width should have been a Len anyway so this will have to change
                let width = width / size[X];

                let aspect = size[X] / size[Y];
                    // Calculate the ring's center and radii
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let outer_radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;
                let inner_radius = outer_radius - width;

                // Check if the mouse is within the ring
                let dx = cursor_pos.0 - center_x;
                let dy = (cursor_pos.1 - center_y) / aspect;
                let distance_squared = dx * dx + dy * dy;
                return distance_squared <= outer_radius * outer_radius
                    && distance_squared >= inner_radius * inner_radius;

            }
        }

    }
}
