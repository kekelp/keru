use glyphon::Cursor as GlyphonCursor;
use glyphon::{Affinity, Resolution as GlyphonResolution};
use rustc_hash::{FxHashMap, FxHasher};

use std::{
    hash::Hasher,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Color as GlyphonColor, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    util::{self, DeviceExt},
    vertex_attr_array, BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device,
    MultisampleState, Queue, RenderPass, RenderPipeline, SurfaceConfiguration, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    keyboard::{ModifiersState, NamedKey},
};
use Axis::{X, Y};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub(crate) u64);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    rect: Xy::new_symm([0.0, 1.0]),
    rect_id: None,
    text_id: None,
    parent_id: NODE_ROOT_ID,
    children_ids: Vec::new(),
    params: NODE_ROOT_PARAMS,
    last_frame_touched: 0,
    last_frame_status: LastFrameStatus::Nothing,
    last_hover: f32::MIN,
    last_click: f32::MIN,
    z: -10000.0,
};

const IX: usize = 0;
const IY: usize = 1;

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
}
impl Axis {
    pub fn other(&self) -> Self {
        match self {
            Axis::X => return Axis::Y,
            Axis::Y => return Axis::X,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Xy<T>([T; 2]);
impl<T> Index<Axis> for Xy<T> {
    type Output = T;
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => return &self.0[0],
            Axis::Y => return &self.0[1],
        }
    }
}
impl<T> IndexMut<Axis> for Xy<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => return &mut self.0[0],
            Axis::Y => return &mut self.0[1],
        }
    }
}
unsafe impl Zeroable for Xy<f32> {}
unsafe impl Pod for Xy<f32> {}

impl<T: Copy> Xy<T> {
    pub const fn new(x: T, y: T) -> Self {
        return Self([x, y]);
    }

    pub const fn new_symm(v: T) -> Self {
        return Self([v, v]);
    }
}

impl Rect {
    pub fn size(&self) -> Xy<f32> {
        return Xy::new(self[X][1] - self[X][0], self[Y][1] - self[Y][0]);
    }
}

// todo: compress some fields... for example, stacks can never be clickable or editable
// maybe remove z as well. nodoby uses that.
#[derive(Debug, Copy, Clone)]
pub struct NodeParams {
    pub debug_name: &'static str,
    pub static_text: Option<&'static str>,
    pub visible_rect: bool,
    pub clickable: bool,
    pub editable: bool,
    pub color: Color,
    pub size: Xy<Size>,
    pub position: Xy<Position>,
    pub is_stack: Option<Stack>,
}

impl Default for NodeParams {
    fn default() -> Self {
        Self {
            debug_name: "DEFAULT",
            static_text: None,
            clickable: false,
            visible_rect: false,
            color: Color::BLUE,
            size: Xy::new_symm(Size::PercentOfAvailable(0.5)),
            position: Xy::new_symm(Position::Start),
            is_stack: None,
            editable: false,
        }
    }
}

impl NodeParams {
    pub const fn const_default() -> Self {
        Self {
            debug_name: "Default Button",
            static_text: Some("Button"),
            clickable: true,
            visible_rect: true,
            color: Color::BLUE,
            size: Xy::new_symm(Size::PercentOfAvailable(0.5)),
            position: Xy::new_symm(Position::Start),
            is_stack: None,
            editable: false,
        }
    }

    pub const fn with_size_x(mut self, size: f32) -> Self {
        self.size.0[IX] = Size::PercentOfAvailable(size);
        return self;
    }
    pub const fn with_size_y(mut self, size: f32) -> Self {
        self.size.0[IY] = Size::PercentOfAvailable(size);
        return self;
    }
    pub const fn with_size_symm(mut self, size: f32) -> Self {
        self.size = Xy::new_symm(Size::PercentOfAvailable(size));
        return self;
    }

    pub const fn with_position_x(mut self, position: Position) -> Self {
        self.position.0[IX] = position;
        return self;
    }
    pub const fn with_position_y(mut self, position: Position) -> Self {
        self.position.0[IY] = position;
        return self;
    }
    pub const fn with_position_symm(mut self, position: Position) -> Self {
        self.position = Xy::new_symm(position);
        return self;
    }

    pub const fn with_static_text(mut self, text: &'static str) -> Self {
        self.static_text = Some(text);
        return self;
    }

    pub const fn with_debug_name(mut self, text: &'static str) -> Self {
        self.debug_name = text;
        return self;
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        return self;
    }

    pub const fn with_stack(mut self, axis: Axis, arrange: Arrange) -> Self {
        self.is_stack = Some(Stack {
            arrange,
            axis,
        });
        return self;
    }

    pub const V_STACK: Self = Self {
        debug_name: "Column",
        static_text: None,
        clickable: true,
        visible_rect: false,
        color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        size: Xy::new(Size::PercentOfAvailable(1.0), Size::PercentOfAvailable(1.0)),
        position: Xy::new_symm(Position::Center),
        is_stack: Some(Stack {
            arrange: Arrange::Start,
            axis: Axis::Y,
        }),
        editable: false,
    };
    pub const H_STACK: Self = Self {
        debug_name: "Column",
        static_text: None,
        visible_rect: false,
        clickable: true,
        color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        size: Xy::new(Size::PercentOfAvailable(1.0), Size::PercentOfAvailable(1.0)),
        position: Xy::new_symm(Position::Center),
        is_stack: Some(Stack {
            arrange: Arrange::Start,
            axis: Axis::X,
        }),
        editable: false,
    };
    pub const FRAME: Self = Self {
        debug_name: "FRAME",
        static_text: None,
        clickable: true,
        visible_rect: false,
        color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        size: Xy::new_symm(Size::PercentOfAvailable(0.7)),
        position: Xy::new_symm(Position::Center),
        is_stack: None,
        editable: false,
    };

    pub const BUTTON: Self = Self {
        debug_name: "Button",
        static_text: None,
        clickable: true,
        visible_rect: true,
        color: Color::rgba(0.0, 0.1, 0.1, 0.9),
        size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
        position: Xy::new_symm(Position::Center),
        is_stack: None,
        editable: false,
    };

    pub const LABEL: Self = Self {
        debug_name: "label",
        static_text: None,
        clickable: false,
        visible_rect: true,
        color: Color::rgba(0.0, 0.1, 0.1, 0.9),
        size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
        position: Xy::new_symm(Position::Center),
        is_stack: None,
        editable: false,
    };

    pub const TEXT_INPUT: Self = Self {
        debug_name: "label",
        static_text: None,
        clickable: true,
        visible_rect: true,
        color: Color::rgba(0.1, 0.0, 0.1, 0.9),
        size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
        position: Xy::new_symm(Position::Start),
        is_stack: None,
        editable: true,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct NodeKey {
    pub id: Id,
    pub defaults: NodeParams,
}

use std::hash::Hash;

impl NodeKey {
    pub const fn new(defaults: NodeParams, id: Id) -> Self {
        return Self { defaults, id };
    }

    pub fn id(&self) -> Id {
        return self.id;
    }

    pub fn with_id(mut self, id: Id) -> Self {
        self.id = id;
        return self;
    }

    // todo: make const?
    // are they really all siblings? the base one is different from all the derived ones.
    pub fn sibling<H: Hash>(&self, value: H) -> Self {
        let mut hasher = FxHasher::default();
        self.id.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            defaults: self.defaults.clone(),
        };
    }

    pub const fn with_defaults(mut self, defaults: NodeParams) -> Self {
        self.defaults = defaults;
        return self;
    }

    // can't use the [X] syntax in const functions: functions in trait impls cannot be declared const
    pub const fn with_size_x(mut self, size: f32) -> Self {
        self.defaults.size.0[IX] = Size::PercentOfAvailable(size);
        return self;
    }
    pub const fn with_size_y(mut self, size: f32) -> Self {
        self.defaults.size.0[IY] = Size::PercentOfAvailable(size);
        return self;
    }
    pub const fn with_size_symm(mut self, size: f32) -> Self {
        self.defaults.size = Xy::new_symm(Size::PercentOfAvailable(size));
        return self;
    }

    pub const fn with_position_x(mut self, position: Position) -> Self {
        self.defaults.position.0[IX] = position;
        return self;
    }
    pub const fn with_position_y(mut self, position: Position) -> Self {
        self.defaults.position.0[IY] = position;
        return self;
    }
    pub const fn with_position_symm(mut self, position: Position) -> Self {
        self.defaults.position = Xy::new_symm(position);
        return self;
    }

    pub const fn with_static_text(mut self, text: &'static str) -> Self {
        self.defaults.static_text = Some(text);
        return self;
    }

    pub const fn with_debug_name(mut self, text: &'static str) -> Self {
        self.defaults.debug_name = text;
        return self;
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.defaults.color = color;
        return self;
    }

    pub const fn with_stack(mut self, axis: Axis, arrange: Arrange) -> Self {
        self.defaults.is_stack = Some(Stack {
            arrange,
            axis,
        });
        return self;
    }
}

#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
#[repr(C)]
// Layout has to match the one in the shader.
pub struct Rectangle {
    // todo: switch to Xy<[f32; 2]>
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,

    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,

    pub last_hover: f32,
    pub last_click: f32,
    pub clickable: u32,
    pub z: f32,

    pub radius: f32,

    // -- useless for shader
    pub _padding: f32,
    pub id: Id,
}
impl Rectangle {
    pub fn buffer_desc() -> [VertexAttribute; 8] {
        return vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32,
            4 => Float32,
            5 => Uint32,
            6 => Float32,
            7 => Float32,
        ];
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    };

    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);

    pub const BLUE: Self = Self {
        r: 0.6,
        g: 0.3,
        b: 1.0,
        a: 0.6,
    };

    pub const LIGHT_BLUE: Self = Self {
        r: 0.9,
        g: 0.7,
        b: 1.0,
        a: 0.6,
    };

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn darken(&mut self, amount: f32) {
        self.r = self.r * (1.0 - amount);
        self.g = self.g * (1.0 - amount);
        self.b = self.b * (1.0 - amount);
        self.a = self.a * (1.0 - amount);
    }

    pub fn lighten(&mut self, amount: f32) {
        self.r = self.r * (1.0 + amount);
        self.g = self.g * (1.0 + amount);
        self.b = self.b * (1.0 + amount);
        self.a = self.a * (1.0 + amount);
    }
}

// a reference to Ui with a particular Id "selected"
// ui.add returns this, so that function calls to update the just-added node are slightly more aesthetic
// this probably counts as oversugaring.
// but with advanced hashmap stuff like RawEntryMut or however the brown is hashed, it should be possible to do something similar to this:
// pub struct UiWithNodeRef<'a> {
//     ui: &'a mut Ui,
//     node: &'a mut Node,
// }
// which would have the same aethetics but would also be faster.

// ^all the above is obsolete. this is great the way it is now. except for the lifetime soup. and it's still oversugaring.
pub struct UiWithNodeKey<'a, 'b> {
    ui: &'a mut Ui,
    key: &'b NodeKey,
}

impl<'a, 'b> UiWithNodeKey<'a, 'b> {
    pub fn set_color(&mut self, color: Color) {
        self.ui.set_color(self.key.id, color)
    }

    pub fn set_text(&mut self, text: &str) {
        self.ui.set_text(&self.key, text)
    }
}

pub struct PartialBorrowStuff {
    pub mouse_pos: PhysicalPosition<f32>,
    pub mouse_left_clicked: bool,
    pub mouse_left_just_clicked: bool,
    pub unifs: Uniforms,
    pub current_frame: u64,
    pub t0: Instant,
}
impl PartialBorrowStuff {
    pub fn is_rect_clicked_or_hovered(&self, rect: &Rectangle) -> (bool, bool) {
        // rects are rebuilt from scratch every render, so this isn't needed, for now. 
        // if rect.last_frame_touched != self.current_frame {
        //     return (false, false);
        // }

        let mut mouse_pos = (
            self.mouse_pos.x / self.unifs.size[X],
            1.0 - (self.mouse_pos.y / self.unifs.size[Y]),
        );

        // transform mouse_pos into "opengl" centered coordinates
        mouse_pos.0 = (mouse_pos.0 * 2.0) - 1.0;
        mouse_pos.1 = (mouse_pos.1 * 2.0) - 1.0;

        let hovered = rect.x0 < mouse_pos.0
            && mouse_pos.0 < rect.x1
            && rect.y0 < mouse_pos.1
            && mouse_pos.1 < rect.y1;

        if hovered == false {
            return (false, false);
        };

        let clicked = self.mouse_left_just_clicked;
        return (clicked, hovered);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BlinkyLine {
    pub index: usize,
    pub affinity: Affinity,
}

#[derive(Debug, Copy, Clone)]
pub enum Cursor {
    BlinkyLine(BlinkyLine),
    Selection((GlyphonCursor, GlyphonCursor)),
}

type Rect = Xy<[f32; 2]>;

#[derive(Debug, Clone, Copy)]
pub enum TreeTraceEntry {
    Node(Id),
    SetParent(Id),
}

pub struct EverythingExceptTreeTrace {}

pub struct Ui {
    pub key_modifiers: ModifiersState,

    pub gpu_vertex_buffer: TypedGpuBuffer<Rectangle>,
    pub render_pipeline: RenderPipeline,

    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: BindGroup,

    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,

    pub rects: Vec<Rectangle>,
    pub text_areas: Vec<TextArea>,
    pub node_map: FxHashMap<Id, Node>,

    // stack for traversing
    pub stack: Vec<Id>,

    // stack for adding
    pub parent_stack: Vec<Id>,

    pub part: PartialBorrowStuff,

    pub clicked_stack: Vec<(Id, f32)>,
    pub hovered_stack: Vec<(Id, f32)>,
    pub clicked: Option<Id>,
    pub hovered: Option<Id>,

    pub focused: Option<Id>,

    // todo: add these back sometime. probably better to have relayout_needed, rerender_needed, etc instead of some vaguely named trash
    // // remember about animations (surely there will be)
    // pub content_changed: bool,
    // pub tree_changed: bool,

    pub t: f32,

    // todo: add this
    // pub last_tree_trace: Vec<TreeTraceEntry>,
    
    pub tree_trace: Vec<TreeTraceEntry>,
    pub tree_trace_defaults: Vec<Option<NodeParams>>,
}
impl Ui {
    pub fn set_text(&mut self, key: &NodeKey, text: &str) {
        let hash = fx_hash(&text);

        if let None = self.node_map.get(&key.id) {
            //todo: call a fake_add that doesn't set parent and children, and remove the whole Option(parent_id) trash everywhere
            self.update_node(key, None);
        }
        let text_id = self.node_map.get(&key.id).unwrap().text_id;
        let text_id = match text_id {
            Some(text_id) => {
                if hash == self.text_areas[text_id].last_hash {
                    // todo: I shouldn't have to do this, I don't think, it's visible as long as the node is visible??
                    self.text_areas[text_id].last_frame_touched = self.part.current_frame;
                    return;
                }
                self.text_areas[text_id].last_hash = hash;

                text_id
            }
            None => {
                let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(42.0, 42.0));
                buffer.set_size(&mut self.font_system, 100000., 100000.);

                let text_area = TextArea {
                    buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 10000,
                        bottom: 10000,
                    },
                    default_color: GlyphonColor::rgb(255, 255, 255),
                    depth: 0.0,
                    last_frame_touched: self.part.current_frame,
                    last_hash: hash,
                };

                self.text_areas.push(text_area);
                let text_id = Some(self.text_areas.len() - 1);
                self.node_map.get_mut(&key.id).unwrap().text_id = text_id;
                text_id.unwrap()
            }
        };
        self.text_areas[text_id].buffer.set_text(
            &mut self.font_system,
            &text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        self.text_areas[text_id].last_frame_touched = self.part.current_frame;
    }

    pub fn set_color(&mut self, id: Id, color: Color) {
        // todo: dont return, add, etc
        if let Some(node) = self.node_map.get_mut(&id) {
            if node.params.color == color {
                return;
            } else {
                node.params.color = color;
            }
        }
    }

    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("player bullet pos buffer"),
            contents: bytemuck::cast_slice(&[0.0; 9000]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vertex_buffer = TypedGpuBuffer::new(vertex_buffer);
        let vert_buff_layout = VertexBufferLayout {
            array_stride: mem::size_of::<Rectangle>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Rectangle::buffer_desc(),
        };

        let uniforms = Uniforms {
            size: Xy::new(config.width as f32, config.height as f32),
            t: 0.,
            _padding: 0.,
        };
        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Resolution Bind Group Layout"),
        });

        // Create the bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: resolution_buffer.as_entire_binding(),
            }],
            label: Some("Resolution Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("box.wgsl").into()),
        });

        let mut primitive = wgpu::PrimitiveState::default();
        primitive.cull_mode = None;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vert_buff_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive,
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let text_areas = Vec::new();

        let mut nodes = FxHashMap::default();

        nodes.insert(NODE_ROOT_ID, NODE_ROOT);

        let mut stack = Vec::with_capacity(7);
        stack.push(NODE_ROOT_ID);

        let mut parent_stack = Vec::with_capacity(7);
        parent_stack.push(NODE_ROOT_ID);

        Self {
            key_modifiers: ModifiersState::default(),
            cache,
            render_pipeline,
            atlas,
            text_renderer,
            font_system,
            text_areas,
            rects: Vec::with_capacity(20),
            node_map: nodes,
            gpu_vertex_buffer: vertex_buffer,
            uniform_buffer: resolution_buffer,
            bind_group,

            stack: Vec::new(),

            parent_stack,

            part: PartialBorrowStuff {
                mouse_pos: PhysicalPosition { x: 0., y: 0. },
                mouse_left_clicked: false,
                mouse_left_just_clicked: false,
                current_frame: 0,
                unifs: uniforms,
                t0: Instant::now(),
            },

            clicked_stack: Vec::new(),
            clicked: None,
            hovered_stack: Vec::new(),
            hovered: None,
            focused: None,

            t: 0.0,

            tree_trace: Vec::new(),
            tree_trace_defaults: Vec::new(),
        }
    }

    pub fn add<'b>(&mut self, node_key: &'b NodeKey) -> UiWithNodeKey<'_, 'b> {
        let id = node_key.id();
        self.tree_trace.push(TreeTraceEntry::Node(id));
        self.tree_trace_defaults.push(Some(node_key.defaults));

        return UiWithNodeKey {
            ui: self,
            key: &node_key,
        };
    }

    // todo: deduplicate with refresh (maybe)
    pub fn update_node<'b>(&mut self, node_key: &'b NodeKey, parent_id: Option<Id>) -> UiWithNodeKey<'_, 'b> {
        let node_key_id = node_key.id;

        let old_node = self.node_map.get_mut(&node_key_id);
        if old_node.is_none() {
            let mut text_id = None;
            if let Some(text) = node_key.defaults.static_text {
                // text size
                let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(42.0, 42.0));
                buffer.set_size(&mut self.font_system, 100000., 100000.);

                let mut hasher = FxHasher::default();
                text.hash(&mut hasher);
                let hash = hasher.finish();

                buffer.set_text(
                    &mut self.font_system,
                    text,
                    Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );

                let text_area = TextArea {
                    buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 10000,
                        bottom: 10000,
                    },
                    default_color: GlyphonColor::rgb(255, 255, 255),
                    depth: 0.0,
                    last_frame_touched: self.part.current_frame,
                    last_hash: hash,
                };
                self.text_areas.push(text_area);
                text_id = Some(self.text_areas.len() - 1);
            }

            let new_node = self.new_node(node_key, parent_id, text_id);
            self.node_map.insert(node_key_id, new_node);
        } else {
            let old_node = old_node.unwrap();
            if let Some(text_id) = old_node.text_id {
                self.text_areas[text_id].last_frame_touched = self.part.current_frame;
            }
            old_node.last_frame_touched = self.part.current_frame;
            old_node.children_ids.clear();
        }


        if let Some(parent_id) = parent_id {
            self.node_map
            .get_mut(&parent_id)
            .unwrap()
            .children_ids
            .push(node_key_id);
        }

        return UiWithNodeKey {
            ui: self,
            key: &node_key,
        };
    }

    pub fn new_node(&self, node_key: &NodeKey, parent_id: Option<Id>, text_id: Option<usize>) -> Node {
        let parent_id = match parent_id {
            Some(parent_id) => parent_id,
            None => Id(999999),
        };
        Node {
            rect_id: None,
            rect: Xy::new_symm([0.0, 1.0]),
            text_id,
            parent_id,
            children_ids: Vec::new(),
            params: node_key.defaults,
            last_frame_touched: self.part.current_frame,
            last_frame_status: LastFrameStatus::Nothing,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
        }
    }

    // pub fn handle_keyboard_event(&mut self, event: &KeyEvent) -> Option<()> {
    //     let id = self.focused?;
    //     let node = self.nodes.get(&id)?;
    //     let text_id = node.text_id?;
    //     // println!(" {:#?}\n", event);

    //     if event.state.is_pressed() {
    //         let buffer = &mut self.text_areas[text_id].buffer;

    //         match &event.logical_key {
    //             winit::keyboard::Key::Named(named_key) => {
    //                 // todo: when holding control all key events arrive duplicated??
    //                 match named_key {
    //                     NamedKey::ArrowLeft => {
    //                         match (
    //                             self.key_modifiers.shift_key(),
    //                             self.key_modifiers.control_key(),
    //                         ) {
    //                             (true, true) => buffer.lines[0].text.control_shift_left_arrow(),
    //                             (true, false) => buffer.lines[0].text.shift_left_arrow(),
    //                             (false, true) => buffer.lines[0].text.control_left_arrow(),
    //                             (false, false) => buffer.lines[0].text.left_arrow(),
    //                         }
    //                     }
    //                     NamedKey::ArrowRight => {
    //                         match (
    //                             self.key_modifiers.shift_key(),
    //                             self.key_modifiers.control_key(),
    //                         ) {
    //                             (true, true) => buffer.lines[0].text.control_shift_right_arrow(),
    //                             (true, false) => buffer.lines[0].text.shift_right_arrow(),
    //                             (false, true) => buffer.lines[0].text.control_right_arrow(),
    //                             (false, false) => buffer.lines[0].text.right_arrow(),
    //                         }
    //                     }
    //                     NamedKey::Backspace => {
    //                         if self.key_modifiers.control_key() {
    //                             buffer.lines[0].text.ctrl_backspace();
    //                         } else {
    //                             buffer.lines[0].text.backspace();
    //                         }
    //                         buffer.lines[0].reset();
    //                     }
    //                     NamedKey::End => {
    //                         match self.key_modifiers.shift_key() {
    //                             true => buffer.lines[0].text.shift_end(),
    //                             false => buffer.lines[0].text.go_to_end(),
    //                         }
    //                         buffer.lines[0].reset();
    //                     }
    //                     NamedKey::Home => {
    //                         match self.key_modifiers.shift_key() {
    //                             false => buffer.lines[0].text.go_to_start(),
    //                             true => buffer.lines[0].text.shift_home(),
    //                         }
    //                         buffer.lines[0].reset();
    //                     }
    //                     NamedKey::Delete => {
    //                         if self.key_modifiers.control_key() {
    //                             buffer.lines[0].text.ctrl_delete();
    //                         } else {
    //                             buffer.lines[0].text.delete();
    //                         }
    //                         buffer.lines[0].reset();
    //                     }
    //                     NamedKey::Space => {
    //                         buffer.lines[0].text.insert_str_at_cursor(" ");
    //                         buffer.lines[0].reset();
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             winit::keyboard::Key::Character(new_char) => {
    //                 buffer.lines[0].text.insert_str_at_cursor(&new_char);
    //                 buffer.lines[0].reset();
    //             }
    //             winit::keyboard::Key::Unidentified(_) => {}
    //             winit::keyboard::Key::Dead(_) => {}
    //         };
    //     }

    //     return Some(());
    // }

    pub fn handle_events(&mut self, full_event: &Event<()>, queue: &Queue) {
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.part.mouse_pos.x = position.x as f32;
                    self.part.mouse_pos.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        if *state == ElementState::Pressed {
                            self.part.mouse_left_clicked = true;
                            if !self.part.mouse_left_just_clicked {
                                self.part.mouse_left_just_clicked = true;
                            }
                        } else {
                            self.part.mouse_left_clicked = false;
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.key_modifiers = modifiers.state();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    // self.handle_keyboard_event(&event);
                }
                WindowEvent::Resized(size) => self.resize(size, queue),
                _ => {}
            }
        }
    }

    pub fn layout(&mut self) {
        self.stack.clear();

        // push the root
        self.stack.push(NODE_ROOT_ID);

        // start processing a parent
        while let Some(current_node_id) = self.stack.pop() {
            // todo: garbage
            let parent_rect: Rect;
            let children: Vec<Id>;
            let is_stack: Option<Stack>;
            {
                let parent_node = self.node_map.get(&current_node_id).unwrap();
                children = parent_node.children_ids.clone();
                parent_rect = parent_node.rect;
                is_stack = parent_node.params.is_stack;
            }
            let parent_size = parent_rect.size();

            match is_stack {
                Some(stack) => {
                    let main_axis = stack.axis;
                    let sign = match stack.arrange {
                        Arrange::Start => 1.0,
                        Arrange::End => -1.0,
                        _ => todo!(),
                    };
                    let i0 = match stack.arrange {
                        Arrange::Start => 0,
                        Arrange::End => 1,
                        _ => todo!(),
                    };
                    let i1 = match stack.arrange {
                        Arrange::Start => 1,
                        Arrange::End => 0,
                        _ => todo!(),
                    };
                    // space for each child on the main axis
                    let n = children.len() as f32;
                    let spacing_pixels = 7;
                    let spacing_f = spacing_pixels as f32 / self.part.unifs.size[main_axis];
                    let main_width = (parent_size[main_axis] - (n - 1.0) * spacing_f as f32) / n;

                    let mut walker = parent_rect[main_axis][i0];

                    for &child_id in children.iter().rev() {
                        let child = self.node_map.get_mut(&child_id).unwrap();
                        child.rect[main_axis][i0] = walker;

                        match child.params.size[main_axis] {
                            Size::PercentOfAvailable(percent) => {
                                let other_corner = walker + sign * main_width * percent;
                                child.rect[main_axis][i1] = other_corner;
                                walker = other_corner + sign * spacing_f;
                            }
                        }

                        let cross_axis = main_axis.other();
                        match child.params.position[cross_axis] {
                            Position::Start => match child.params.size[cross_axis] {
                                Size::PercentOfAvailable(percent) => {
                                    let cross_0 = parent_rect[cross_axis][0]
                                        + spacing_f;
                                    let cross_1 = cross_0 + parent_size[cross_axis] * percent;
                                    child.rect[cross_axis] = [cross_0, cross_1];
                                }
                            },
                            Position::Center => match child.params.size[cross_axis] {
                                Size::PercentOfAvailable(percent) => {
                                    let center =
                                        parent_rect[cross_axis][0] + parent_size[cross_axis] / 2.0;
                                    let width = parent_size[cross_axis] * percent;
                                    let x0 = center - width / 2.0;
                                    let x1 = center + width / 2.0;
                                    child.rect[cross_axis] = [x0, x1];
                                }
                            },
                            Position::End => todo!(),
                        }

                        let rect = child.rect;
                        let text_id = child.text_id;
                        self.layout_text(text_id, rect);

                        self.stack.push(child_id);
                    }
                }
                None => {
                    for &child_id in children.iter().rev() {
                        let child = self.node_map.get_mut(&child_id).unwrap();

                        for axis in [X, Y] {
                            match child.params.position[axis] {
                                Position::Start => {
                                    let x0 = parent_rect[axis][0];
                                    match child.params.size[axis] {
                                        Size::PercentOfAvailable(percent) => {
                                            let x1 = x0 + parent_size[axis] * percent;
                                            child.rect[axis] = [x0, x1];
                                        }
                                    }
                                }
                                Position::End => {
                                    let x1 = parent_rect[axis][1];
                                    match child.params.size[axis] {
                                        Size::PercentOfAvailable(percent) => {
                                            let x0 = x1 - parent_size[axis] * percent;
                                            child.rect[axis] = [x0, x1];
                                        }
                                    }
                                }
                                Position::Center => {
                                    let center = parent_rect[axis][0] + parent_size[axis] / 2.0;
                                    match child.params.size[axis] {
                                        Size::PercentOfAvailable(percent) => {
                                            let width = parent_size[axis] * percent;
                                            let x0 = center - width / 2.0;
                                            let x1 = center + width / 2.0;
                                            child.rect[axis] = [x0, x1];
                                        }
                                    }
                                }
                            }
                        }

                        // trash language
                        let rect = child.rect;
                        let text_id = child.text_id;
                        self.layout_text(text_id, rect);

                        self.stack.push(child_id);
                    }
                }
            }
        }
    }

    pub fn layout_text(&mut self, text_id: Option<usize>, rect: Rect) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].left = rect[X][0] * self.part.unifs.size[X];
            self.text_areas[text_id].top = (1.0 - rect[Y][1]) * self.part.unifs.size[Y];
            self.text_areas[text_id]
                .buffer
                .shape_until_scroll(&mut self.font_system, false);
        }
    }

    pub fn is_clicked(&self, id: Id) -> bool {
        if !self.part.mouse_left_just_clicked {
            return false;
        }
        if let Some(clicked_id) = &self.clicked {
            return *clicked_id == id;
        }
        return false;
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        self.part.unifs.size[X] = size.width as f32;
        self.part.unifs.size[Y] = size.height as f32;

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            &bytemuck::bytes_of(&self.part.unifs)[..16],
        );
    }

    pub fn update_gpu_time(&mut self, queue: &Queue) {
        // magical offset...
        queue.write_buffer(&self.uniform_buffer, 8, bytemuck::bytes_of(&self.t));
    }

    pub fn update_time(&mut self) {
        self.t = self.part.t0.elapsed().as_secs_f32();
    }

    pub fn build_buffers(&mut self) {

        self.rects.clear();
        self.stack.clear();

        // push the ui.direct children of the root without processing the root
        if let Some(root) = self.node_map.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter().rev() {
                self.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.stack.pop() {
            let current_node = self.node_map.get_mut(&current_node_id).unwrap();

            if current_node.params.visible_rect
                && current_node.last_frame_touched == self.part.current_frame
            {
                self.rects.push(Rectangle {
                    x0: current_node.rect[X][0] * 2. - 1.,
                    x1: current_node.rect[X][1] * 2. - 1.,
                    y0: current_node.rect[Y][0] * 2. - 1.,
                    y1: current_node.rect[Y][1] * 2. - 1.,
                    r: current_node.params.color.r,
                    g: current_node.params.color.g,
                    b: current_node.params.color.b,
                    a: current_node.params.color.a,
                    last_hover: current_node.last_hover,
                    last_click: current_node.last_click,
                    clickable: current_node.params.clickable.into(),
                    id: current_node_id,
                    z: 0.0,
                    radius: 30.0,
                    _padding: 0.0,
                });
            }

            for &child_id in current_node.children_ids.iter() {
                self.stack.push(child_id);
            }
        }

        // self.push_cursor_rect();
    }

    // pub fn push_cursor_rect(&mut self) -> Option<()> {
    //     // cursor
    //     // how to make it appear at the right z? might be impossible if there are overlapping rects at the same z.
    //     // one epic way could be to increase the z sequentially when rendering, so that all rects have different z's, so the cursor can have the z of its rect plus 0.0001.
    //     // would definitely be very cringe for anyone doing custom rendering. but not really. nobody will ever want to stick his custom rendered stuff between a rectangle and another. when custom rendering INSIDE a rectangle, the user can get the z every time. might be annoying (very annoying even) but not deal breaking.

    //     // it's a specific choice by me to keep cursors for every string at all times, but only display (and use) the one on the currently focused ui node.
    //     // someone might want multi-cursor in the same node, multi-cursor on different nodes, etc.
    //     let focused_id = &self.focused?;
    //     let focused_node = self.nodes.get(focused_id)?;
    //     let text_id = focused_node.text_id?;
    //     let focused_text_area = self.text_areas.get(text_id)?;

    //     match focused_text_area.buffer.lines[0].text.cursor() {
    //         StringCursor::Point(cursor) => {
    //             let rect_x0 = focused_node.rect[X][0];
    //             let rect_y1 = focused_node.rect[Y][1];

    //             let (x, y) = cursor_pos_from_byte_offset(&focused_text_area.buffer, *cursor);

    //             let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
    //             let cursor_height = focused_text_area.buffer.metrics().font_size;
    //             // we're counting on this always happening after layout. which should be safe.
    //             let x0 = ((x - 1.0) / self.part.unifs.size[X]) * 2.0;
    //             let x1 = ((x + cursor_width) / self.part.unifs.size[X]) * 2.0;
    //             let x0 = x0 + (rect_x0 * 2. - 1.);
    //             let x1 = x1 + (rect_x0 * 2. - 1.);

    //             let y0 = ((-y - cursor_height) / self.part.unifs.size[Y]) * 2.0;
    //             let y1 = ((-y) / self.part.unifs.size[Y]) * 2.0;
    //             let y0 = y0 + (rect_y1 * 2. - 1.);
    //             let y1 = y1 + (rect_y1 * 2. - 1.);

    //             let cursor_rect = Rectangle {
    //                 x0,
    //                 x1,
    //                 y0,
    //                 y1,
    //                 r: 0.5,
    //                 g: 0.3,
    //                 b: 0.5,
    //                 a: 0.9,
    //                 last_hover: 0.0,
    //                 last_click: 0.0,
    //                 clickable: 0,
    //                 z: 0.0,
    //                 id: Id(0),
    //                 _padding: 0.0,
    //                 radius: 0.0,
    //             };

    //             self.rects.push(cursor_rect);
    //         }
    //         StringCursor::Selection(selection) => {
    //             let rect_x0 = focused_node.rect[X][0];
    //             let rect_y1 = focused_node.rect[Y][1];

    //             let (x0, y0) =
    //                 cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.start);
    //             let (x1, y1) =
    //                 cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.end);

    //             // let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
    //             let cursor_height = focused_text_area.buffer.metrics().font_size;
    //             let x0 = ((x0 - 1.0) / self.part.unifs.size[X]) * 2.0;
    //             let x1 = ((x1 + 1.0) / self.part.unifs.size[X]) * 2.0;
    //             let x0 = x0 + (rect_x0 * 2. - 1.);
    //             let x1 = x1 + (rect_x0 * 2. - 1.);

    //             let y0 = ((-y0 - cursor_height) / self.part.unifs.size[Y]) * 2.0;
    //             let y1 = ((-y1) / self.part.unifs.size[Y]) * 2.0;
    //             let y0 = y0 + (rect_y1 * 2. - 1.);
    //             let y1 = y1 + (rect_y1 * 2. - 1.);

    //             let cursor_rect = Rectangle {
    //                 x0,
    //                 x1,
    //                 y0,
    //                 y1,
    //                 r: 0.5,
    //                 g: 0.3,
    //                 b: 0.5,
    //                 a: 0.9,
    //                 last_hover: 0.0,
    //                 last_click: 0.0,
    //                 clickable: 0,
    //                 z: 0.0,
    //                 id: Id(0),
    //                 _padding: 0.0,
    //                 radius: 0.0,
    //             };

    //             self.rects.push(cursor_rect);
    //         }
    //     }

    //     return Some(());
    // }

    pub fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let n = self.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.text_renderer.render(&self.atlas, render_pass).unwrap();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        self.gpu_vertex_buffer.queue_write(&self.rects[..], queue);

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                GlyphonResolution {
                    width: self.part.unifs.size[X] as u32,
                    height: self.part.unifs.size[Y] as u32,
                },
                &mut self.text_areas,
                &mut self.cache,
                self.part.current_frame,
            )
            .unwrap();
    }

    pub fn begin_tree(&mut self) {
        self.update_time();

        self.tree_trace.clear();
        self.tree_trace_defaults.clear();

        self.node_map
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();

        self.part.current_frame += 1;
    }

    pub fn finish_tree(&mut self) {
        self.update_nodes();
        self.layout();
        self.resolve_mouse_input();
        
    }

    // todo: skip this if there has been no new mouse movement and no new clicks.
    pub fn resolve_mouse_input(&mut self) {
        self.clicked_stack.clear();
        self.hovered_stack.clear();
        self.hovered = None;
        self.clicked = None;

        for rect in &self.rects {
            if rect.clickable != 0 {
                let (clicked, hovered) = self.part.is_rect_clicked_or_hovered(&rect);
                if clicked {
                    self.clicked_stack.push((rect.id, rect.z));
                } else if hovered {
                    self.hovered_stack.push((rect.id, rect.z));
                }
            }
        }

        // only the one with the highest z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        // there may be exceptions.

        let mut max_z = f32::MAX;
        for (id, z) in self.clicked_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                self.clicked = Some(*id);
            }
        }

        let mut max_z = f32::MAX;
        for (id, z) in self.hovered_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                self.hovered = Some(*id);
            }
        }

        // this goes on the node because the rect isn't a real entity. it's rebuilt every frame
        if let Some(id) = self.hovered {
            let node = self.node_map.get_mut(&id).unwrap();
            node.last_hover = self.t;
        }

        let mut focused_anything = false;
        if let Some(id) = self.clicked {
            let node = self.node_map.get_mut(&id).unwrap();
            node.last_click = self.t;

            if node.params.editable {
                self.focused = self.clicked;
                focused_anything = true;
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.text_areas[id];
                let (x, y) = (
                    self.part.mouse_pos.x - text_area.left,
                    self.part.mouse_pos.y - text_area.top,
                );

                // todo: with how I'm misusing cosmic-text, this might become "unsafe" soon (as in, might be incorrect or cause panics, not actually unsafe).
                // I think in general, there should be a safe version of hit() that just forces a rerender just to be sure that the offset is safe to use.
                // But in this case, calling this in resolve_mouse_input() and not on every winit mouse event probably means it's safe

                // actually, the enlightened way is that cosmic_text exposes an "unsafe" hit(), but we only ever see the string + cursor + buffer struct, and call that hit(), which doesn't return an offset but just mutates the one inside.
                text_area.buffer.hit(x, y);
            }
        }

        // defocus when use clicked anywhere else
        if self.part.mouse_left_just_clicked && focused_anything == false {
            self.focused = None;
        }
    }

    pub fn start_layer(&mut self, parent_id: Id) {
        self.parent_stack.push(parent_id);
        self.tree_trace.push(TreeTraceEntry::SetParent(parent_id));
        self.tree_trace_defaults.push(None);
    }

    pub fn end_layer(&mut self) {
        self.parent_stack.pop();
        let new_parent = self.parent_stack.last().unwrap();
        self.tree_trace.push(crate::ui::TreeTraceEntry::SetParent(*new_parent));
        self.tree_trace_defaults.push(None);
    }

    pub(crate) fn update_nodes(&mut self) {
        let mut current_parent_id = NODE_ROOT_ID;
        for i in 0..self.tree_trace.len() {
            match &self.tree_trace[i] {
                TreeTraceEntry::Node(id) => {
                    let defaults = self.tree_trace_defaults[i].unwrap();
                    let key = NodeKey::new(defaults, *id);
                    self.update_node(&key, Some(current_parent_id));
                },
                TreeTraceEntry::SetParent(id) => {
                    current_parent_id = *id;
                },
            }
        }
    }
}

// todo: since macros = le bad, maybe make separate functions so that it's possible to do 
// ui.begin_hstack()
// ui.add(children)
// ui.end_hstack()
// multiple ways to do the same thing = also le bad albeit
macro_rules! create_layer_macro {
    ($macro_name:ident, $node_params_name:expr) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($ui:expr, $code:block) => {
                let anonymous_id = new_id!();
                let node_key = NodeKey::new($node_params_name, anonymous_id);
                $ui.add(&node_key);

                $ui.start_layer(anonymous_id);
                
                $code;
                
                $ui.end_layer();
            };
            // named
            ($ui:expr, $node_key:expr, $code:block) => {
                $ui.add($node_key);

                $ui.start_layer($node_key.id);
                
                $code;
                
                $ui.end_layer();
            };
        }
    };
}

create_layer_macro!(h_stack, NodeParams::H_STACK);
create_layer_macro!(v_stack, NodeParams::V_STACK);
create_layer_macro!(margin, NodeParams::FRAME);

#[derive(Debug)]
pub struct Node {
    // visible rect only
    pub rect_id: Option<usize>,
    // also for invisible rects, used for layout
    pub rect: Xy<[f32; 2]>,

    pub last_frame_touched: u64,
    pub last_frame_status: LastFrameStatus,

    pub text_id: Option<usize>,

    pub parent_id: Id,
    // todo: maybe switch with that prev/next thing
    pub children_ids: Vec<Id>,
    pub params: NodeParams,

    pub last_hover: f32,
    pub last_click: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastFrameStatus {
    Clicked,
    Hovered,
    Nothing,
}

#[derive(Debug, Clone, Copy)]
pub enum Len {
    PercentOfParent(f32),
    Pixels(u32),
}
impl Len {
    pub fn to_pixels(&self, parent_pixels: u32) -> u32 {
        match self {
            Len::PercentOfParent(percent) => return (parent_pixels as f32 * percent) as u32,
            Len::Pixels(pixels) => return pixels.clone(),
        }
    }
}

// textorimagecontent is more of a "min size" thing, I think.
#[derive(Debug, Clone, Copy)]
pub enum Size {
    PercentOfAvailable(f32),
    // Pixels(u32),
    // TextOrImageContent { padding: u32 },
    // // ImageContent { padding: u32 },
    // FillParent { padding: u32 },
    // // SumOfChildren { padding: u32 },
    // TrustParent,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Center,
    Start,
    End,
    // TrustParent,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Stack {
    arrange: Arrange,
    axis: Axis,
}

#[derive(Debug, Clone, Copy)]
pub enum Arrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

pub const NODE_ROOT_KEY: NodeKey = NodeKey {
    id: NODE_ROOT_ID,
    defaults: NODE_ROOT_PARAMS,
};

pub const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    debug_name: "ROOT",
    static_text: None,
    visible_rect: false,
    clickable: false,
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    },
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Start),
    is_stack: None,
    editable: false,
};

// todo: change
// copied from stackoverflow: https://stackoverflow.com/questions/71463576/
pub const fn callsite_hash(
    module_path: &'static str,
    filename: &'static str,
    line: u32,
    column: u32,
) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    let prime = 0x00000100000001B3;

    let mut i = 0;

    let mut bytes = module_path.as_bytes();
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    bytes = filename.as_bytes();
    i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    hash ^= line as u64;
    hash = hash.wrapping_mul(prime);
    hash ^= column as u64;
    hash = hash.wrapping_mul(prime);
    return hash;
}

#[macro_export]
macro_rules! new_id {
    () => {{
        $crate::Id($crate::ui::callsite_hash(
            std::module_path!(),
            std::file!(),
            std::line!(),
            std::column!(),
        ))
    }};
}

#[macro_export]
macro_rules! unique_node_key {
    () => {{
        let id = $crate::Id($crate::ui::callsite_hash(
            std::module_path!(),
            std::file!(),
            std::line!(),
            std::column!(),
        ));
        NodeKey {
            defaults: NodeParams::const_default(),
            id,
        }
    }};
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

#[derive(Debug)]
pub struct TypedGpuBuffer<T: Pod> {
    pub buffer: wgpu::Buffer,
    pub marker: std::marker::PhantomData<T>,
}
impl<T: Pod> TypedGpuBuffer<T> {
    pub fn new(buffer: wgpu::Buffer) -> Self {
        Self {
            buffer,
            marker: PhantomData::<T>,
        }
    }

    pub fn size() -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn slice<N: Into<u64>>(&self, n: N) -> wgpu::BufferSlice {
        let bytes = n.into() * (mem::size_of::<T>()) as u64;
        return self.buffer.slice(..bytes);
    }

    pub fn queue_write(&mut self, data: &[T], queue: &Queue) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, data);
    }
}

pub fn cursor_pos_from_byte_offset(buffer: &Buffer, byte_offset: usize) -> (f32, f32) {
    let line = &buffer.lines[0];
    let buffer_line = line.layout_opt().as_ref().unwrap();
    let glyphs = &buffer_line[0].glyphs;

    // todo: binary search? lol. maybe vec has it built in
    for g in glyphs {
        if g.start >= byte_offset {
            return (g.x, g.y);
        }
    }

    if let Some(glyph) = glyphs.last() {
        return (glyph.x + glyph.w, glyph.y);
    }

    // string is empty
    return (0.0, 0.0);
}

fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}