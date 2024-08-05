use crate::node_params::{DEFAULT, NODE_ROOT_PARAMS};
use crate::unwrap_or_return;
use copypasta::{ClipboardContext, ClipboardProvider};
use glyphon::cosmic_text::{Align, StringCursor};
use glyphon::{AttrsList, Cursor as GlyphonCursor};
use glyphon::{Affinity, Resolution as GlyphonResolution};
use rustc_hash::{FxHashMap, FxHasher};
use slab::Slab;
use wgpu::*;
use winit::keyboard::Key;

use crate::for_each_child;

use std::collections::hash_map::Entry;
use std::ops::{Add, Mul, Sub};
use std::{
    hash::Hasher,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Color as GlyphonColor, Family, FontSystem, Metrics, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyEvent, MouseButton, WindowEvent},
    keyboard::{ModifiersState, NamedKey},
};
use Axis::{X, Y};
use {
    util::{self, DeviceExt},
    vertex_attr_array, BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device,
    MultisampleState, Queue, RenderPass, RenderPipeline, SurfaceConfiguration, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub(crate) u64);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    id: NODE_ROOT_ID,
    rect: Xy::new_symm([0.0, 1.0]),
    size: Xy::new_symm(10.0),
    rect_id: None,
    text_id: None,
    // geeeeeeeeeeeeg wtf
    parent: unsafe {
        std::mem::transmute(9213432846u64)
    },

    n_children: 0,
    first_child: None,
    prev_sibling: None,
    next_sibling: None,

    params: NODE_ROOT_PARAMS,
    last_frame_status: LastFrameStatus::Nothing,
    last_hover: f32::MIN,
    last_click: f32::MIN,
    z: -10000.0,
};

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

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Xy<T> {
    pub x: T,
    pub y: T,
}

impl<T: Add<Output = T> + Copy> Add<Xy<T>> for Xy<T> {
    type Output = Self;
    fn add(self, rhs: Xy<T>) -> Self::Output {
        let new_x = self.x + rhs.x;
        let new_y = self.y + rhs.y;
        return Self::new(new_x, new_y);
    }
}
impl<T: Sub<Output = T> + Copy> Sub<Xy<T>> for Xy<T> {
    type Output = Self;
    fn sub(self, rhs: Xy<T>) -> Self::Output {
        let new_x = self.x - rhs.x;
        let new_y = self.y - rhs.y;
        return Self::new(new_x, new_y);
    }
}
impl<T: Add<Output = T> + Copy> Add<(T, T)> for Xy<T> {
    type Output = Self;
    fn add(self, rhs: (T, T)) -> Self::Output {
        let new_x = self.x + rhs.0;
        let new_y = self.y + rhs.1;
        return Self::new(new_x, new_y);
    }
}

impl<T> Index<Axis> for Xy<T> {
    type Output = T;
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => return &self.x,
            Axis::Y => return &self.y,
        }
    }
}
impl<T> IndexMut<Axis> for Xy<T> {
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => return &mut self.x,
            Axis::Y => return &mut self.y,
        }
    }
}
unsafe impl Zeroable for Xy<f32> {}
unsafe impl Pod for Xy<f32> {}
unsafe impl Zeroable for Xy<[f32; 2]> {}
unsafe impl Pod for Xy<[f32; 2]> {}

impl<T: Copy> Xy<T> {
    pub const fn new(x: T, y: T) -> Self {
        return Self { x, y };
    }

    pub const fn new_symm(v: T) -> Self {
        return Self { x: v, y: v };
    }
}

type Rect = Xy<[f32; 2]>;

impl Rect {
    pub fn size(&self) -> Xy<f32> {
        return Xy::new(self[X][1] - self[X][0], self[Y][1] - self[Y][0]);
    }

    pub fn new2(origin: Xy<f32>, size: Xy<f32>) -> Self {
        return Self {
            x: [origin.x, origin.x + size.x],
            y: [origin.y, origin.y + size.y]
        }
    }
}
impl Add<f32> for Rect {
    type Output = Self;
    fn add(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] + rhs, self[X][1] + rhs],
            [self[Y][0] + rhs, self[Y][1] + rhs],
        );
    }
}
impl Sub<f32> for Rect {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] - rhs, self[X][1] - rhs],
            [self[Y][0] - rhs, self[Y][1] - rhs],
        );
    }
}
impl Mul<f32> for Rect {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        return Self::new(
            [self[X][0] * rhs, self[X][1] * rhs],
            [self[Y][0] * rhs, self[Y][1] * rhs],
        );
    }
}

// todo: compress some fields... for example, stacks can never be clickable or editable
#[derive(Debug, Copy, Clone)]
pub struct NodeParams {
    pub static_text: Option<&'static str>,
    pub visible_rect: bool,
    pub clickable: bool,
    pub editable: bool,
    pub filled: bool,
    pub color: Color,
    pub size: Xy<Size>,
    pub position: Xy<Position>,
    pub stack: Option<Stack>,
    #[cfg(debug_assertions)]
    pub debug_name: &'static str,
}

impl NodeParams {
    pub const fn const_default() -> Self {
        DEFAULT
    }

    pub const fn size_x(mut self, size: Len) -> Self {
        self.size.x = Size::Fixed(size);
        return self;
    }
    pub const fn size_y(mut self, size: Len) -> Self {
        self.size.y = Size::Fixed(size);
        return self;
    }
    pub const fn size_symm(mut self, size: Len) -> Self {
        self.size = Xy::new_symm(Size::Fixed(size));
        return self;
    }

    pub const fn position_x(mut self, position: Position) -> Self {
        self.position.x = position;
        return self;
    }
    pub const fn position_y(mut self, position: Position) -> Self {
        self.position.y = position;
        return self;
    }
    pub const fn position_symm(mut self, position: Position) -> Self {
        self.position = Xy::new_symm(position);
        return self;
    }

    pub const fn text(mut self, text: &'static str) -> Self {
        self.static_text = Some(text);
        return self;
    }

    #[cfg(debug_assertions)]
    pub const fn debug_name(mut self, text: &'static str) -> Self {
        self.debug_name = text;
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        return self;
    }

    pub const fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange) -> Self {
        self.stack = Some(Stack { arrange, axis });
        return self;
    }
}

#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
#[repr(C)]
// todo: could do some epic SOA stuff to make resolve_mouse_input and friends faster
//   could also store the pre-transformed coordinates
// Layout has to match the one in the shader.
pub struct RenderRect {
    pub rect: Rect,

    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,

    pub last_hover: f32,
    pub last_click: f32,
    pub clickable: u32,
    pub z: f32,

    pub radius: f32,

    // -- not used in shader
    pub filled: u32,
    pub id: Id,
}
impl RenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 9] {
        return vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32,
            4 => Float32,
            5 => Uint32,
            6 => Float32,
            7 => Float32,
            8 => Uint32,
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

    pub const BLUE: Self = Self::rgba(0.1, 0.1, 1.0, 0.6);
    pub const RED: Self = Self::rgba(1.0, 0.1, 0.1, 0.6);
    pub const GREEN: Self = Self::rgba(0.1, 1.0, 0.1, 0.6);

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
        self.r *= 1.0 - amount;
        self.g *= 1.0 - amount;
        self.b *= 1.0 - amount;
        self.a *= 1.0 - amount;
    }

    pub fn lighten(&mut self, amount: f32) {
        self.r *= 1.0 + amount;
        self.g *= 1.0 + amount;
        self.b *= 1.0 + amount;
        self.a *= 1.0 + amount;
    }
}

pub struct NodeWithStuff<'a> {
    node: &'a mut Node,
    text: &'a mut Text,
}

// why can't you just do it separately?
impl<'a> NodeWithStuff<'a> {
    pub fn set_color(&mut self, color: Color)  -> &mut Self {
        self.node.params.color = color;
        return self;
    }

    pub fn set_text(&mut self, text: &str) -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.text.set_text(text_id, text);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }

        return self;
    }

    pub fn set_text_attrs(&mut self, attrs: Attrs)  -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.text.set_text_attrs(text_id, attrs);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn set_text_align(&mut self, align: Align)  -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.text.set_text_align(text_id, align);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn get_text(&mut self) -> Option<String> {
        let text_id = self.node.text_id.unwrap();

        let text = self.text.text_areas[text_id].buffer.lines[0]
            .text
            .text()
            .to_string();
        return Some(text);
    }
}

pub struct PartialBorrowStuff {
    pub mouse_pos: PhysicalPosition<f32>,
    pub unifs: Uniforms,
    pub current_frame: u64,
    pub t0: Instant,
}
impl PartialBorrowStuff {
    pub fn mouse_hit_rect(&self, rect: &RenderRect) -> bool {
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

        let hovered = rect.rect[X][0] < mouse_pos.0
            && mouse_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < mouse_pos.1
            && mouse_pos.1 < rect.rect[Y][1];

        return hovered;
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

// another stupid sub struct for dodging partial borrows
pub struct Text {
    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_areas: Vec<TextArea>,
}
const GLOBAL_TEXT_METRICS: Metrics = Metrics::new(24.0, 24.0);
impl Text {
    pub fn new_text_area(&mut self, text: Option<&str>, current_frame: u64) -> Option<usize> {
        let text = text?;

        let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
        buffer.set_size(&mut self.font_system, 500., 500.);

        let mut hasher = FxHasher::default();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );


        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Center));
        }

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
            last_frame_touched: current_frame,
            last_hash: hash,
        };
        self.text_areas.push(text_area);
        let text_id = self.text_areas.len() - 1;

        return Some(text_id);
    }

    fn refresh_last_frame(&mut self, text_id: Option<usize>, current_frame: u64) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].last_frame_touched = current_frame;
        }
    }

    fn set_text(&mut self, text_id: usize, text: &str) {
        let hash = fx_hash(&text);
        let area = &mut self.text_areas[text_id];
        if hash != area.last_hash {
            area.last_hash = hash;
            area.buffer.set_text(
                &mut self.font_system,
                text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
        }
    }

    fn set_text_attrs(&mut self, text_id: usize, attrs: Attrs) {

        let area = &mut self.text_areas[text_id];

        // Define new attributes
        // Apply new attributes to the entire text
        for line in &mut area.buffer.lines {
            line.set_attrs_list(AttrsList::new(attrs));
        }

    }

    fn set_text_align(&mut self, text_id: usize, align: Align) {

        for line in &mut self.text_areas[text_id].buffer.lines {
            line.set_align(Some(align));
        }

    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Idx(pub(crate) u64);

#[derive(Debug, Clone, Copy)]
pub struct NodeFront {
    pub last_parent: usize,
    pub last_frame_touched: u64,
    
    // keeping track of the twin situation. This is the number of twins of a node that showed up SO FAR in the current frame. it gets reset every frame (on refresh().)
    // for the 0-th twin of a family, this will be the total number of clones of itself around. (not including itself, so starts at zero).
    // the actual twins ARE twins, but they don't HAVE twins, so this is zero.
    // for this reason, "clones" or "copies" would be better names, but those words are loaded in rust
    // reproduction? replica? imitation? duplicate? version? dupe? replication? mock? carbon?    
    pub n_twins: u32,
    pub slab_i: usize,
}
impl NodeFront {
    pub fn new(parent_id: usize, frame: u64, new_i: usize) -> Self {
        return Self {
            last_parent: parent_id,
            last_frame_touched: frame,
            n_twins: 0,
            slab_i: new_i,
        }
    }

    pub fn refresh(&mut self, parent_id: usize, frame: u64) {
        self.last_frame_touched = frame;
        self.last_parent = parent_id;
        self.n_twins = 0;
    }


}

pub struct Nodes {
    // todo: make faster o algo
    pub fronts: FxHashMap<Id, NodeFront>,
    pub nodes: Slab<Node>,
}
impl Nodes {
    pub fn get_by_id(&mut self, id: &Id) -> Option<&mut Node> {
        let i = self.fronts.get(&id).unwrap().slab_i;
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

pub struct Ui {
    pub root_i: usize,
    pub debug_mode: bool,
    pub debug_key_pressed: bool,

    pub waiting_for_click_release: bool,

    pub clipboard: ClipboardContext,

    pub key_mods: ModifiersState,

    pub gpu_vertex_buffer: TypedGpuBuffer<RenderRect>,
    pub render_pipeline: RenderPipeline,

    pub base_uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub text: Text,

    pub rects: Vec<RenderRect>,

    pub nodes: Nodes,
    
    // stack for traversing
    pub traverse_stack: Vec<usize>,

    // stack for adding
    pub parent_stack: Vec<usize>,

    // head of the sibling linked list or somehting
    // todo2: check if this always get unwrapped, maybe remove the option
    pub last_child_stack: Vec<usize>,

    pub part: PartialBorrowStuff,

    pub clicked_stack: Vec<(Id, f32)>,
    pub mouse_hit_stack: Vec<(Id, f32)>,
    pub clicked: Vec<Id>,
    pub hovered: Vec<Id>,

    pub focused: Option<Id>,

    // todo: add these back sometime. probably better to have relayout_needed, rerender_needed, etc instead of some vaguely named trash
    // // remember about animations (surely there will be)
    // pub content_changed: bool,
    // pub tree_changed: bool,
    pub t: f32,
}
impl Ui {

    pub fn to_pixels_axis(&self, len: Len) -> u32 {
        match len {
            Len::Pixels(pixels) => return pixels,
            Len::Frac(frac) => return (frac * self.part.unifs.size.x) as u32,
        }
    }

    pub fn to_frac_axis(&self, len: Len, axis: Axis) -> f32 {
        match len {
            Len::Pixels(pixels) => return (pixels as f32) / self.part.unifs.size[axis],
            Len::Frac(frac) => return frac,
        }
    }

    pub fn to_frac2(&self, len: Xy<Len>) -> Xy<f32> {
        return Xy::new(
            self.to_frac_axis(len.x, X),
            self.to_frac_axis(len.y, Y),
        );
    }

    fn instant_t(&self) -> f32 {
        return self.part.t0.elapsed().as_secs_f32();
    }

    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("player bullet pos buffer"),
            contents: bytemuck::cast_slice(&[0.0; 9000]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vertex_buffer = TypedGpuBuffer::new(vertex_buffer);
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Resolution Bind Group Layout"),
        });

        // Create the bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: resolution_buffer.as_entire_binding(),
            }],
            label: Some("Resolution Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("box.wgsl").into()),
        });

        let mut primitive = PrimitiveState::default();
        primitive.cull_mode = None;

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vert_buff_layout],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let text_areas = Vec::with_capacity(50);

        let mut node_fronts = FxHashMap::with_capacity_and_hasher(100, Default::default());
        

        
        let mut nodes = Slab::with_capacity(100);
        let root_i = nodes.insert(NODE_ROOT);
        let root_nodefront = NodeFront {
            last_parent: usize::default(),
            last_frame_touched: u64::MAX,
            slab_i: root_i,
            n_twins: 0,
        };
        
        let mut stack = Vec::with_capacity(7);
        stack.push(root_i);
        
        let mut parent_stack = Vec::with_capacity(7);
        parent_stack.push(root_i);

        node_fronts.insert(NODE_ROOT_ID, root_nodefront);

        let nodes = Nodes {
            fronts: node_fronts,
            nodes,
        };

        Self {
            root_i,
            waiting_for_click_release: false,
            debug_mode: false,
            debug_key_pressed: false,
            clipboard: ClipboardContext::new().unwrap(),
            key_mods: ModifiersState::default(),

            text: Text {
                cache,
                atlas,
                text_renderer,
                font_system,
                text_areas,
            },

            render_pipeline,
            rects: Vec::with_capacity(20),

            nodes,

            gpu_vertex_buffer: vertex_buffer,
            base_uniform_buffer: resolution_buffer,
            bind_group,

            traverse_stack: Vec::with_capacity(50),

            parent_stack,

            last_child_stack: Vec::with_capacity(20),

            part: PartialBorrowStuff {
                mouse_pos: PhysicalPosition { x: 0., y: 0. },
                current_frame: 1,
                unifs: uniforms,
                t0: Instant::now(),
            },

            clicked_stack: Vec::with_capacity(50),
            mouse_hit_stack: Vec::with_capacity(50),
            clicked: Vec::with_capacity(15),
            hovered: Vec::with_capacity(15),
            focused: None,

            t: 0.0,
        }
    }

    pub fn add(&mut self, key: NodeKey) -> NodeWithStuff {
        return self.update_node(key, false);
    }

    pub fn add_layer(&mut self, key: NodeKey) -> NodeWithStuff {
        return self.update_node(key, true);
    }

    pub fn end_layer(&mut self) {
        self.parent_stack.pop();
        self.last_child_stack.pop();
    }

    pub fn update_node(&mut self, key: NodeKey, make_new_layer: bool) -> NodeWithStuff {
        let parent_id = self.parent_stack.last().unwrap().clone();

        let frame = self.part.current_frame;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame: 
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.fronts.entry(key.id()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {
                let text_id = self.text.new_text_area(key.defaults().static_text, frame);
                let new_node = Self::new_node(&key, Some(parent_id), text_id);
                
                let final_i = self.nodes.nodes.insert(new_node);
                v.insert(NodeFront::new(parent_id, frame, final_i));

                UpdatedNormal{ final_i }
            },
            Entry::Occupied(o) => {
                let old_nodefront = o.into_mut();
                
                match refresh_or_add_twin(frame, old_nodefront.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_nodefront.refresh(parent_id, frame);
                        // todo2: check the nodefront values and maybe skip reaching into the node
                        let final_i = old_nodefront.slab_i;
                        self.refresh_node(final_i, parent_id, frame);
                        UpdatedNormal{ final_i }
                    }
                    // do nothing, just calculate the twin key and go to twin part below
                    AddTwin => {
                        old_nodefront.n_twins += 1;
                        let twin_key = key.sibling(old_nodefront.n_twins);
                        NeedToUpdateTwin { twin_key }
                    }
                }

            },
        };

        // If twin_check_result is AddedNormal, the node was added in the section before, 
        //      and there's nothing to do regarding twins, so we just confirm final_i.
        // If it's NeedToAddTwin, we repeat the same thing with the new twin_key.
        let real_final_i = match twin_check_result {
            UpdatedNormal { final_i } => final_i,
            NeedToUpdateTwin { twin_key } => {
                match self.nodes.fronts.entry(twin_key.id()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let text_id = self.text.new_text_area(twin_key.defaults().static_text, frame);
                        let new_twin_node = Self::new_node(&twin_key, Some(parent_id), text_id);
    
                        let real_final_i = self.nodes.nodes.insert(new_twin_node);
                        v.insert(NodeFront::new(parent_id, frame, real_final_i));
                        real_final_i
                    },
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_nodefront = o.into_mut();
    
                        // todo2: check the nodefront values and maybe skip reaching into the node
                        old_twin_nodefront.refresh(parent_id, frame);
    
                        let real_final_i = old_twin_nodefront.slab_i;
                        self.refresh_node(real_final_i, parent_id, frame);
                        real_final_i
                    },
    
                }
            },
        };

        self.add_child_to_parent(real_final_i, parent_id);
        if make_new_layer {
            self.parent_stack.push(real_final_i);           
        }

        return NodeWithStuff {
            node: &mut self.nodes[real_final_i],
            text: &mut self.text,
        };

    }

    pub fn add_child_to_parent(&mut self, id: usize, parent_id: usize) {
        self.nodes[parent_id].n_children += 1;

        if self.nodes[parent_id].first_child == None {
            self.nodes[parent_id].first_child = Some(id);

            self.last_child_stack.push(id);

        } else {
            let prev_sibling = *self.last_child_stack.last().unwrap();
            self.nodes[id].prev_sibling = Some(prev_sibling);
            self.nodes[prev_sibling].next_sibling = Some(id);
            *self.last_child_stack.last_mut().unwrap() = id;
        }


    }

    // todo: why like this
    pub fn new_node(
        key: &NodeKey,
        parent_id: Option<usize>,
        text_id: Option<usize>,
    ) -> Node {
        let parent_id = match parent_id {
            Some(parent_id) => parent_id,
            None => usize::default(),
        };
        Node {
            id: key.id(),
            rect_id: None,
            rect: Xy::new_symm([0.0, 1.0]),
            size: Xy::new_symm(10.0),
            text_id,
            parent: parent_id,

            n_children: 0,
            first_child: None,
            prev_sibling: None,
            next_sibling: None,
        
            params: key.defaults(),
            last_frame_status: LastFrameStatus::Nothing,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        // todo: remove line.reset(); and do it only once per frame via change watcher guy

        match &event.logical_key {
            Key::Named(named_key) => match named_key {
                NamedKey::F1 => {
                    if event.state.is_pressed() {
                        if self.debug_key_pressed == false {
                            #[cfg(debug_assertions)]
                            {
                                self.debug_mode = !self.debug_mode;
                            }
                        }
                    }

                    self.debug_key_pressed = event.state.is_pressed();
                }
                _ => {}
            },
            _ => {}
        }

        // if there is no focused text node, return consumed: false
        let id = unwrap_or_return!(self.focused, false);
        let node = unwrap_or_return!(self.nodes.get_by_id(&id), false);
        let text_id = unwrap_or_return!(node.text_id, false);

        // return consumed: true in each of these cases. Still don't consume keys that the UI doesn't use.
        if event.state.is_pressed() {
            let buffer = &mut self.text.text_areas[text_id].buffer;
            let line = &mut buffer.lines[0];

            match &event.logical_key {
                // todo: ctrl + Z
                Key::Named(named_key) => match named_key {
                    NamedKey::ArrowLeft => {
                        match (self.key_mods.shift_key(), self.key_mods.control_key()) {
                            (true, true) => line.text.control_shift_left_arrow(),
                            (true, false) => line.text.shift_left_arrow(),
                            (false, true) => line.text.control_left_arrow(),
                            (false, false) => line.text.left_arrow(),
                        }
                        return true;
                    }
                    NamedKey::ArrowRight => {
                        match (self.key_mods.shift_key(), self.key_mods.control_key()) {
                            (true, true) => line.text.control_shift_right_arrow(),
                            (true, false) => line.text.shift_right_arrow(),
                            (false, true) => line.text.control_right_arrow(),
                            (false, false) => line.text.right_arrow(),
                        }
                        return true;
                    }
                    NamedKey::Backspace => {
                        if self.key_mods.control_key() {
                            line.text.ctrl_backspace();
                        } else {
                            line.text.backspace();
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::End => {
                        match self.key_mods.shift_key() {
                            true => line.text.shift_end(),
                            false => line.text.go_to_end(),
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Home => {
                        match self.key_mods.shift_key() {
                            false => line.text.go_to_start(),
                            true => line.text.shift_home(),
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Delete => {
                        if self.key_mods.control_key() {
                            line.text.ctrl_delete();
                        } else {
                            line.text.delete();
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Space => {
                        line.text.insert_str_at_cursor(" ");
                        line.reset();
                        return true;
                    }
                    _ => {}
                },
                Key::Character(new_char) => {
                    if !self.key_mods.control_key()
                        && !self.key_mods.alt_key()
                        && !self.key_mods.super_key()
                    {
                        line.text.insert_str_at_cursor(new_char);
                        line.reset();
                        return true;
                    } else if self.key_mods.control_key() {
                        match new_char.as_str() {
                            "c" => {
                                let selected_text = line.text.selected_text().to_owned();
                                if let Some(text) = selected_text {
                                    let _ = self.clipboard.set_contents(text.to_string());
                                }
                                return true;
                            }
                            "v" => {
                                if let Ok(pasted_text) = self.clipboard.get_contents() {
                                    line.text.insert_str_at_cursor(&pasted_text);
                                    line.reset();
                                }
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
                Key::Unidentified(_) => {}
                Key::Dead(_) => {}
            };
        }

        return false;
    }

    // returns: is the event consumed?
    pub fn handle_events(&mut self, full_event: &Event<()>, queue: &Queue) -> bool {
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.part.mouse_pos.x = position.x as f32;
                    self.part.mouse_pos.y = position.y as f32;
                    self.resolve_hover();
                    // cursormoved is never consumed
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        let is_pressed = state.is_pressed();
                        if is_pressed {
                            let consumed = self.resolve_click();
                            return consumed;
                        } else {
                            let waiting_for_click_release = self.waiting_for_click_release;
                            let on_rect = self.resolve_click_release();
                            let consumed = on_rect && waiting_for_click_release;
                            return consumed;
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.key_mods = modifiers.state();
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic,
                    ..
                } => {
                    if !is_synthetic {
                        let consumed = self.handle_keyboard_event(event);
                        return consumed;
                    }
                }
                WindowEvent::Resized(size) => self.resize(size, queue),
                _ => {}
            }
        }

        return false;
    }

    pub fn layout2(&mut self) {
        self.determine_size(self.root_i, Xy::new(1.0, 1.0));
        self.place_children(self.root_i, Xy::new(0.0, 0.0));
    }

    fn determine_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        self.nodes[node].size = match self.nodes[node].params.stack {
            Some(stack) => {
                self.determine_size_stack(node, proposed_size, stack)
            }
            None => {
                self.determine_size_normal(node, proposed_size)
            }
        };
        return self.nodes[node].size;
    }

    fn determine_size_stack(&mut self, node: usize, proposed_size: Xy<f32>, stack: Stack) -> Xy<f32> {
        let (main, cross) = (stack.axis, stack.axis.other());
        // container. this should look a lot different: the size_per_child can decrease or increase if the first child ends up taking more/less than proposed
        
        let mut child_proposed_size = Xy::new(0.0, 0.0);
        let n_children = self.nodes[node].n_children as f32;
        child_proposed_size[main] = proposed_size[main] / n_children;
        child_proposed_size[cross] = proposed_size[cross];

        let padding = self.to_frac2(self.nodes[node].params.size.padding());
        child_proposed_size[main] += 2.0 * padding[main] * n_children;

        let mut final_self_size = Xy::new(0.0, 0.0);

        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.determine_size(child, child_proposed_size);
            
            final_self_size[main] += child_size[main];
            if child_size[cross] > final_self_size[cross] {
                final_self_size[cross] = child_size[cross];
            }
        });

        return final_self_size;
    }

    fn determine_size_normal(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let mut final_size = proposed_size;
        let mut biggest_child_size = Xy::new(0.0, 0.0);
        for_each_child!(self, self.nodes[node], child, {
            self.determine_size(child, proposed_size);
            for axis in [X, Y] {
                if self.nodes[child].size[axis] > biggest_child_size[axis] {
                    biggest_child_size[axis] = self.nodes[child].size[axis];
                }
            }
        });

        for axis in [X, Y] {
            match self.nodes[node].params.size[axis] {
                Size::Fill { .. } => {
                    // leave proposed_size 
                },
                Size::JustAsBigAsBiggestChild { padding } => {
                    // dumb double loop
                    let padding = self.to_frac_axis(padding, axis);
                    final_size[axis] = biggest_child_size[axis] + 2.0 * padding;
                },
                Size::Fixed(len) => {
                    final_size[axis] = self.to_frac_axis(len, axis);
                },
                Size::TextContent { .. } => {
                    const TEXT_SIZE_LOL: Xy<f32> = Xy::new(0.15, 0.066);
                    final_size[axis] = TEXT_SIZE_LOL[axis];
                },
            }
        }

        return final_size;
    }

    fn place_children(&mut self, node: usize, origin: Xy<f32>) {

        if let Some(stack) = self.nodes[node].params.stack {

            let main = stack.axis;

            let mut current_origin = origin;
            
            for_each_child!(self, self.nodes[node], child, {
                let size = self.nodes[child].size;
                let padding = self.nodes[node].params.size.padding();
                let padding = self.to_frac2(padding);
                let child_origin = current_origin + padding;
                
                self.nodes[child].rect = Rect::new2(child_origin, size);

                self.place_children(child, child_origin);

                current_origin[main] += self.nodes[child].size[main] + padding[main];
            });

        } else {
            for_each_child!(self, self.nodes[node], child, {
                let size = self.nodes[child].size;
                let padding = self.to_frac2(self.nodes[node].params.size.padding());
                let child_origin = origin + padding;

                self.nodes[child].rect = Rect::new2(child_origin, size);

                self.place_children(child, child_origin);
            });
        }

        self.layout_text(self.nodes[node].text_id, self.nodes[node].rect);

        println!(" place  : {:?} = {:?}", self.nodes[node].params.debug_name, self.nodes[node].rect);

    }

    // pub fn layout(&mut self) {
    //     self.traverse_stack.clear();
    //     // push the root
    //     self.traverse_stack.push(self.root_i);

    //     // start processing a parent
    //     while let Some(parent) = self.traverse_stack.pop() {
            
    //         let n = self.nodes[parent].n_children as f32;
    //         let parent_rect = self.nodes[parent].rect;

    //         match self.nodes[parent].params.stack {
    //             Some(stack) => {
    //                 let main_axis = stack.axis;
    //                 let sign = match stack.arrange {
    //                     Arrange::Start => 1.0,
    //                     Arrange::End => -1.0,
    //                     _ => todo!(),
    //                 };
    //                 let i0 = match stack.arrange {
    //                     Arrange::Start => 0,
    //                     Arrange::End => 1,
    //                     _ => todo!(),
    //                 };
    //                 let i1 = match stack.arrange {
    //                     Arrange::Start => 1,
    //                     Arrange::End => 0,
    //                     _ => todo!(),
    //                 };
    //                 // space for each child on the main axis
    //                 let spacing_pixels = 7;
    //                 let spacing_f = spacing_pixels as f32 / self.part.unifs.size[main_axis];
    //                 let main_width = (parent_rect.size()[main_axis] - (n - 1.0) * spacing_f) / n;

    //                 let mut walker = parent_rect[main_axis][i0];

    //                 let mut current_child_i = self.nodes[parent].first_child;
    //                 while let Some(child_i) = current_child_i {

    //                     let child = &mut self.nodes[child_i];
    //                     child.rect[main_axis][i0] = walker;

    //                     match child.params.size[main_axis] {
    //                         Size::PercentOfAvailable(percent) => {
    //                             let other_corner = walker + sign * main_width * percent;
    //                             child.rect[main_axis][i1] = other_corner;
    //                             walker = other_corner + sign * spacing_f;
    //                         }
    //                     }

    //                     let cross_axis = main_axis.other();
    //                     match child.params.position[cross_axis] {
    //                         Position::Start => match child.params.size[cross_axis] {
    //                             Size::PercentOfAvailable(percent) => {
    //                                 let cross_0 = parent_rect[cross_axis][0] + spacing_f;
    //                                 let cross_1 = cross_0 + parent_rect.size()[cross_axis] * percent;
    //                                 child.rect[cross_axis] = [cross_0, cross_1];
    //                             }
    //                         },
    //                         Position::Center => match child.params.size[cross_axis] {
    //                             Size::PercentOfAvailable(percent) => {
    //                                 let center =
    //                                     parent_rect[cross_axis][0] + parent_rect.size()[cross_axis] / 2.0;
    //                                 let width = parent_rect.size()[cross_axis] * percent;
    //                                 let x0 = center - width / 2.0;
    //                                 let x1 = center + width / 2.0;
    //                                 child.rect[cross_axis] = [x0, x1];
    //                             }
    //                         },
    //                         Position::End => todo!(),
    //                     }

    //                     let rect = child.rect;
    //                     let text_id = child.text_id;
    //                     self.layout_text(text_id, rect);

    //                     self.traverse_stack.push(child_i);
    //                     current_child_i = self.nodes[child_i].next_sibling;
    //                 }
    //             }
    //             None => {
    //                 let mut current_child_i = self.nodes[parent].first_child;
    //                 while let Some(child_i) = current_child_i {

    //                     let child = &mut self.nodes[child_i];

    //                     for axis in [X, Y] {
    //                         match child.params.position[axis] {
    //                             Position::Start => {
    //                                 let x0 = parent_rect[axis][0];
    //                                 match child.params.size[axis] {
    //                                     Size::PercentOfAvailable(percent) => {
    //                                         let x1 = x0 + parent_rect.size()[axis] * percent;
    //                                         child.rect[axis] = [x0, x1];
    //                                     }
    //                                 }
    //                             }
    //                             Position::End => {
    //                                 let x1 = parent_rect[axis][1];
    //                                 match child.params.size[axis] {
    //                                     Size::PercentOfAvailable(percent) => {
    //                                         let x0 = x1 - parent_rect.size()[axis] * percent;
    //                                         child.rect[axis] = [x0, x1];
    //                                     }
    //                                 }
    //                             }
    //                             Position::Center => {
    //                                 let center = parent_rect[axis][0] + parent_rect.size()[axis] / 2.0;
    //                                 match child.params.size[axis] {
    //                                     Size::PercentOfAvailable(percent) => {
    //                                         let width = parent_rect.size()[axis] * percent;
    //                                         let x0 = center - width / 2.0;
    //                                         let x1 = center + width / 2.0;
    //                                         child.rect[axis] = [x0, x1];
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }

    //                     // trash language
    //                     let rect = child.rect;
    //                     let text_id = child.text_id;
    //                     self.layout_text(text_id, rect);

    //                     self.traverse_stack.push(child_i);
    //                     current_child_i = self.nodes[child_i].next_sibling;

    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn layout_text(&mut self, text_id: Option<usize>, rect: Rect) {
        if let Some(text_id) = text_id {
            let left = rect[X][0] * self.part.unifs.size[X];
            let top = (1.0 - rect[Y][1]) * self.part.unifs.size[Y];

            let right = rect[X][1] * self.part.unifs.size[X];
            let bottom = (1.0 - rect[Y][0]) * self.part.unifs.size[Y];

            self.text.text_areas[text_id].left = left;
            self.text.text_areas[text_id].top = top;
           
            self.text.text_areas[text_id].bounds.left = left as i32;
            self.text.text_areas[text_id].bounds.top = top as i32;

            self.text.text_areas[text_id].bounds.right = right as i32;
            self.text.text_areas[text_id].bounds.bottom = bottom as i32;

            let w = right - left;
            let h = bottom - top;
            self.text.text_areas[text_id].buffer.set_size(&mut self.text.font_system, w, h);
           
            self.text.text_areas[text_id]
                .buffer
                .shape_until_scroll(&mut self.text.font_system, false);
        }
    }

    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        return self.clicked.contains(&node_key.id);
    }

    // todo: is_clicked_advanced

    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.hovered.last() != Some(&node_key.id);
    }

    // todo: is_hovered_advanced

    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        self.part.unifs.size[X] = size.width as f32;
        self.part.unifs.size[Y] = size.height as f32;

        queue.write_buffer(
            &self.base_uniform_buffer,
            0,
            &bytemuck::bytes_of(&self.part.unifs)[..16],
        );
    }

    pub fn update_time(&mut self) {
        self.t = self.part.t0.elapsed().as_secs_f32();
    }

    pub fn build_buffers(&mut self) {
        self.rects.clear();
        self.traverse_stack.clear();

        // push the ui.direct children of the root without processing the root
        let mut current_child = self.nodes[self.root_i].first_child;
        while let Some(child) = current_child {
            self.traverse_stack.push(child);
            current_child = self.nodes[child].next_sibling;
        }

        while let Some(node) = self.traverse_stack.pop() {
            let current_node = self.nodes.nodes.get(node).unwrap();

            // in debug mode, draw invisible rects as well.
            // usually these have filled = false (just the outline), but this is not enforced.
            if current_node.params.visible_rect || self.debug_mode {
                self.rects.push(RenderRect {
                    rect: current_node.rect * 2. - 1.,

                    r: current_node.params.color.r,
                    g: current_node.params.color.g,
                    b: current_node.params.color.b,
                    a: current_node.params.color.a,
                    last_hover: current_node.last_hover,
                    last_click: current_node.last_click,
                    clickable: current_node.params.clickable.into(),
                    id: current_node.id,
                    z: 0.0,
                    radius: 30.0,
                    filled: current_node.params.filled as u32,
                });
            }


            let mut current_child = current_node.first_child;
            while let Some(child) = current_child {
                self.traverse_stack.push(child);
                current_child = self.nodes[child].next_sibling;
            }
        }

        println!("len {:?}", self.nodes.nodes.len());
        self.push_cursor_rect();
    }

    pub fn push_cursor_rect(&mut self) -> Option<()> {
        // cursor
        // how to make it appear at the right z? might be impossible if there are overlapping rects at the same z.
        // one epic way could be to increase the z sequentially when rendering, so that all rects have different z's, so the cursor can have the z of its rect plus 0.0001.
        // would definitely be very cringe for anyone doing custom rendering. but not really. nobody will ever want to stick his custom rendered stuff between a rectangle and another. when custom rendering INSIDE a rectangle, the user can get the z every time. might be annoying (very annoying even) but not deal breaking.

        // it's a specific choice by me to keep cursors for every string at all times, but only display (and use) the one on the currently focused ui node.
        // someone might want multi-cursor in the same node, multi-cursor on different nodes, etc.
        let focused_id = &self.focused?;
        let focused_node = self.nodes.get_by_id(focused_id)?;
        let text_id = focused_node.text_id?;
        let focused_text_area = self.text.text_areas.get(text_id)?;

        match focused_text_area.buffer.lines[0].text.cursor() {
            StringCursor::Point(cursor) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];

                let (x, y) = cursor_pos_from_byte_offset(&focused_text_area.buffer, *cursor);

                let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                // we're counting on this always happening after layout. which should be safe.
                let x0 = ((x - 1.0) / self.part.unifs.size[X]) * 2.0;
                let x1 = ((x + cursor_width) / self.part.unifs.size[X]) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);

                let y0 = ((-y - cursor_height) / self.part.unifs.size[Y]) * 2.0;
                let y1 = ((-y) / self.part.unifs.size[Y]) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);

                let cursor_rect = RenderRect {
                    rect: Rect::new([x0, x1], [y0, y1]),
                    r: 0.5,
                    g: 0.3,
                    b: 0.5,
                    a: 0.9,
                    last_hover: 0.0,
                    last_click: 0.0,
                    clickable: 0,
                    z: 0.0,
                    id: Id(0),
                    filled: 1,
                    radius: 0.0,
                };

                self.rects.push(cursor_rect);
            }
            StringCursor::Selection(selection) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];

                let (x0, y0) =
                    cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.start);
                let (x1, y1) =
                    cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.end);

                // let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                let x0 = ((x0 - 1.0) / self.part.unifs.size[X]) * 2.0;
                let x1 = ((x1 + 1.0) / self.part.unifs.size[X]) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);

                let y0 = ((-y0 - cursor_height) / self.part.unifs.size[Y]) * 2.0;
                let y1 = ((-y1) / self.part.unifs.size[Y]) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);

                let cursor_rect = RenderRect {
                    rect: Rect::new([x0, x1], [y0, y1]),

                    r: 0.5,
                    g: 0.3,
                    b: 0.5,
                    a: 0.9,
                    last_hover: 0.0,
                    last_click: 0.0,
                    clickable: 0,
                    z: 0.0,
                    id: Id(0),
                    filled: 1,
                    radius: 0.0,
                };

                self.rects.push(cursor_rect);
            }
        }

        return Some(());
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let n = self.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.text
            .text_renderer
            .render(&self.text.atlas, render_pass)
            .unwrap();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {       
        
        // self.prune();
        self.build_buffers();
        self.gpu_vertex_buffer.queue_write(&self.rects[..], queue);
        
        // update gpu time
        // magical offset...
        queue.write_buffer(&self.base_uniform_buffer, 8, bytemuck::bytes_of(&self.t));

        self.text
            .text_renderer
            .prepare(
                device,
                queue,
                &mut self.text.font_system,
                &mut self.text.atlas,
                GlyphonResolution {
                    width: self.part.unifs.size[X] as u32,
                    height: self.part.unifs.size[Y] as u32,
                },
                &mut self.text.text_areas,
                &mut self.text.cache,
                self.part.current_frame,
            )
            .unwrap();

        // do cleanup here????
        self.hovered.clear();
        self.clicked.clear()
    }

    pub fn begin_tree(&mut self) {
        // do cleanup here??

        self.update_time();

        self.nodes[self.root_i].reset_children();

        self.part.current_frame += 1;
    }

    pub fn finish_tree(&mut self) {
        self.layout2();
        self.resolve_hover();
    }

    pub fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.mouse_hit_stack.clear();

        for rect in &self.rects {
            if rect.clickable != 0 {
                if self.part.mouse_hit_rect(rect) {
                    self.mouse_hit_stack.push((rect.id, rect.z));
                }
            }
        }

        // only the one with the highest z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut max_z = f32::MAX;
        for (id, z) in self.mouse_hit_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
    }

    // called on every mouse movement AND on every frame.
    pub fn resolve_hover(&mut self) {
        let topmost_mouse_hit = self.scan_mouse_hits();

        if let Some(hovered_id) = topmost_mouse_hit {
            self.hovered.push(hovered_id);
            // this goes on the node because the rect isn't a real entity. it's rebuilt every frame
            // todo: if that ever changes, this could skip the hashmap access and get faster, I think.
            let t = self.instant_t();
            let node = self.nodes.get_by_id(&hovered_id).unwrap();
            node.last_hover = t;
        }
    }

    pub fn resolve_click(&mut self) -> bool {
        let topmost_mouse_hit = self.scan_mouse_hits();

        // defocus when use clicking anywhere outside.
        self.focused = None;

        if let Some(clicked_id) = topmost_mouse_hit {
            self.waiting_for_click_release = true;

            self.clicked.push(clicked_id);
            // this goes on the node because the rect isn't a real entity. it's rebuilt every frame
            // todo: if that ever changes, this could skip the hashmap access and get faster, I think.
            let t = self.instant_t();
            let node = self.nodes.get_by_id(&clicked_id).unwrap();
            node.last_click = t;

            if node.params.editable {
                self.focused = Some(clicked_id);
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.text.text_areas[id];
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

        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn resolve_click_release(&mut self) -> bool {
        self.waiting_for_click_release = false;
        let topmost_mouse_hit = self.scan_mouse_hits();
        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn set_text(&mut self, key: NodeKey, text: &str) {
        if let Some(node) = self.nodes.get_by_id(&key.id()) {
            let text_id = node.text_id.unwrap();
            self.text.set_text(text_id, text);
        }
    }

    pub fn prune(&mut self) {
        self.nodes.fronts.retain( |k, v| {
            // the > is to always keep the root node without having to refresh it 
            let should_retain = v.last_frame_touched >= self.part.current_frame;
            // dbg!(k, v.clone());
            // dbg!(to_prune, v.last_frame_touched, self.part.current_frame);
            // println!(" " );
            if ! should_retain {
                // side effect happens inside this closure... weird
                self.nodes.nodes.remove(v.slab_i);
                println!(" PRUNING {:?} {:?}", k, v);
            }
            should_retain
        });
    }

    fn refresh_node(&mut self, final_i: usize, parent_id: usize, frame: u64) {
        let old_node = &mut self.nodes[final_i];
                        
        old_node.refresh(parent_id);
        self.text.refresh_last_frame(old_node.text_id, frame);
    }
}

#[macro_export]
macro_rules! add {
    ($ui:expr, $node_key:expr, $code:block) => {
        $ui.add_layer($node_key);
        $code;
        $ui.end_layer();
    };
    ($ui:expr, $node_key:expr) => {
        $ui.add($node_key)
    };
}

macro_rules! create_layer_macro {
    ($macro_name:ident, $node_params_name:expr) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($ui:expr, $code:block) => {
                let anonymous_key = view_derive::anon_node_key!($node_params_name);
                $ui.add_layer(anonymous_key);
                $code;
                $ui.end_layer();
            };

            // named version. allows writing this: h_stack!(ui, CUSTOM_H_STACK, { ... })
            // it's basically the same as add!, not sure if it's even worth having.
            // especially with no checks that CUSTOM_H_STACK is actually a h_stack.
            ($ui:expr, $node_key:expr, $code:block) => {
                $ui.add_layer($node_key);
                $code;
                $ui.end_layer();
            };
        }
    };
}

create_layer_macro!(h_stack, crate::node_params::H_STACK);
create_layer_macro!(v_stack, crate::node_params::V_STACK);
create_layer_macro!(margin, crate::node_params::MARGIN);
create_layer_macro!(panel, crate::node_params::PANEL);

#[macro_export]
macro_rules! text {
    ($ui:expr, $text:expr) => {
        let anonymous_key = view_derive::anon_node_key!(crate::node_params::TEXT);
        $ui.add(anonymous_key).set_text($text);
    };
}

#[macro_export]
macro_rules! tree {
    ($ui:expr, $code:block) => {{
        $ui.begin_tree();
        $code;
        $ui.finish_tree();
    }};
}

// todo: a lot of the stuff in NodeParams isn't really needed again after creating the node.
// probably only the layout stuff is needed.
#[derive(Debug)]
pub struct Node {
    pub id: Id,
    // visible rect only
    pub rect_id: Option<usize>,
    // also for invisible rects, used for layout
    pub rect: Rect,

    // partial result when layouting?
    pub size: Xy<f32>,

    pub last_frame_status: LastFrameStatus,

    pub text_id: Option<usize>,

    pub parent: usize,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    pub n_children: u16,
    pub first_child: Option<usize>,
    pub prev_sibling: Option<usize>,
    pub next_sibling: Option<usize>,

    pub params: NodeParams,

    pub last_hover: f32,
    pub last_click: f32,
    pub z: f32,
}
impl Node {
    fn reset_children(&mut self) {
        self.first_child = None;
        self.n_children = 0;
    }

    fn refresh(&mut self, parent_id: usize) {
        self.parent = parent_id;
        self.reset_children();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastFrameStatus {
    Clicked,
    Hovered,
    Nothing,
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Fixed(Len),
    Fill {
        padding: Len,
    },
    TextContent {
        padding: Len
        // something like "strictness":
        //  with the "proposed" thing, a TextContent can either insist to get the minimum size it wants,
        // or be okay with whatever (and clip it, show some "..."'s, etc) 
    },
    JustAsBigAsBiggestChild {
        padding: Len,
    }
    // todo: add JustAsBigAsBiggestChildInitiallyButNeverResizeAfter 
}
impl Size {
    pub fn padding(&self) -> Len {
        match self {
            Size::Fixed(_) => Len::ZERO,
            Size::Fill { padding } => *padding,
            Size::TextContent { padding } => *padding,
            Size::JustAsBigAsBiggestChild { padding } => *padding,
        }
    }
}
impl Xy<Size> {
    pub fn padding(&self) -> Xy<Len> {
        return Xy::new(
            self.x.padding(),
            self.y.padding(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Len {
    Pixels(u32),
    Frac(f32),
}
impl Len {
    pub const ZERO: Self = Self::Frac(0.0);
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
    pub arrange: Arrange,
    pub axis: Axis,
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

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

#[derive(Debug)]
pub struct TypedGpuBuffer<T: Pod> {
    pub buffer: Buffer,
    pub marker: std::marker::PhantomData<T>,
}
impl<T: Pod> TypedGpuBuffer<T> {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            marker: PhantomData::<T>,
        }
    }

    pub fn size() -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn slice<N: Into<u64>>(&self, n: N) -> BufferSlice {
        let bytes = n.into() * (mem::size_of::<T>()) as u64;
        return self.buffer.slice(..bytes);
    }

    pub fn queue_write(&mut self, data: &[T], queue: &Queue) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, data);
    }
}

pub fn cursor_pos_from_byte_offset(buffer: &GlyphonBuffer, byte_offset: usize) -> (f32, f32) {
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

use std::hash::Hash;
fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

// #[derive(Debug, Default)]
// pub struct MouseButtons {
//     pub left: bool,
//     pub right: bool,
//     pub middle: bool,
//     pub back: bool,
//     pub forward: bool,
//     pub other: u16, // 16-bit field for other buttons
// }
// impl MouseButtons {
//     pub fn is_other_button_pressed(&self, id: u16) -> bool {
//         if id < 16 {
//             return self.other & (1 << id) != 0;
//         } else {
//             panic!("Mouse button id must be between 0 and 15")
//         }
//     }
// }

// #[derive(Debug)]
// pub struct FilteredMouseInput {
//     pub position: PhysicalPosition<f64>,
//     pub buttons: MouseButtons,
//     pub scroll_delta: (f32, f32),
// }

// impl Default for FilteredMouseInput {
//     fn default() -> Self {
//         return Self {
//             position: PhysicalPosition::new(0.0, 0.0),
//             buttons: MouseButtons::default(),
//             scroll_delta: (0.0, 0.0),
//         };
//     }
// }

// impl FilteredMouseInput {

//     pub fn update(&mut self, event: &WindowEvent) {
//         match event {
//             WindowEvent::CursorMoved { position, .. } => {
//                 self.position = *position;
//             }
//             WindowEvent::MouseInput { state, button, .. } => {
//                 let pressed = *state == ElementState::Pressed;
//                 match button {
//                     MouseButton::Left => self.buttons.left = pressed,
//                     MouseButton::Right => self.buttons.right = pressed,
//                     MouseButton::Middle => self.buttons.middle = pressed,
//                     MouseButton::Back => self.buttons.back = pressed,
//                     MouseButton::Forward => self.buttons.forward = pressed,
//                     MouseButton::Other(id) => {
//                         if *id < 16 {
//                             if pressed {
//                                 self.buttons.other |= 1 << id;
//                             } else {
//                                 self.buttons.other &= !(1 << id);
//                             }
//                         }
//                     }
//                 }
//             }
//             WindowEvent::MouseWheel { delta, .. } => {
//                 match delta {
//                     MouseScrollDelta::LineDelta(x, y) => {
//                         self.scroll_delta.0 += x;
//                         self.scroll_delta.1 += y;
//                     }
//                     MouseScrollDelta::PixelDelta(pos) => {
//                         self.scroll_delta.0 += pos.x as f32;
//                         self.scroll_delta.1 += pos.y as f32;
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }

//     pub fn reset_scroll(&mut self) {
//         self.scroll_delta = (0.0, 0.0);
//     }
// }

#[macro_export]
macro_rules! unwrap_or_return {
    ($expression:expr, $return_value:tt $(,)?) => {{
        match $expression {
            None => return $return_value,
            Some(val) => val,
        }
    }};
}

#[derive(Clone, Copy, Debug)]
pub struct NodeKey {
    defaults: &'static NodeParams,
    id: Id,
}

impl NodeKey {
    pub fn id(&self) -> Id {
        return self.id;
    }
    pub fn defaults(&self) -> NodeParams {
        return *self.defaults;
    }
    pub const fn new(params: &'static NodeParams, id: Id) -> Self {
        return Self { defaults: params, id };
    }
    pub fn sibling<T: Hash>(self, value: T) -> Self {
        let mut hasher = FxHasher::default();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return NodeKey {
            defaults: self.defaults,
            id: Id(new_id),
        };
    }
}

use RefreshOrClone::*;
pub enum RefreshOrClone {
    Refresh,
    AddTwin,
}
pub fn refresh_or_add_twin(current_frame: u64, old_node_last_frame_touched: u64) -> RefreshOrClone {
    if current_frame == old_node_last_frame_touched {
        // re-adding something that got added in the same frame: making a clone
        return RefreshOrClone::AddTwin;
    } else {
        // re-adding something that was added in an old frame: just a normal refresh
        return RefreshOrClone::Refresh;
    }
}

use TwinCheckResult::*;
enum TwinCheckResult {
    UpdatedNormal {
        final_i: usize,
    },
    NeedToUpdateTwin {
        twin_key: NodeKey,
    }
}

#[macro_export]
// todo: use this macro everywhere else
macro_rules! for_each_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.nodes[$child].next_sibling;
            }
        }
    };
}
