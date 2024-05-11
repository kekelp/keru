use glyphon::{cosmic_text::{Scroll, StringCursor}, Affinity, Resolution as GlyphonResolution};
use rustc_hash::{FxHashMap, FxHasher};

use glyphon::{Cursor as GlyphonCursor};
use unicode_segmentation::UnicodeSegmentation;
use std::{future::pending, hash::Hasher, marker::PhantomData, mem, ops::{Index, IndexMut}, time::Instant};

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
    dpi::{PhysicalPosition, PhysicalSize}, event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent}, keyboard::{ModifiersState, NamedKey}
};
use Axis::{X, Y};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub(crate) u64);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    rect: Xy::new_symm([99999.0, 99999.0]),
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
impl<T: Copy> Xy<T> {
    pub const fn new(x: T, y: T) -> Self {
        return Self([x, y]);
    }

    pub const fn new_symm(v: T) -> Self {
        return Self([v, v]);
    }
}

#[derive(Debug, Clone)]
pub struct NodeParams {
    pub debug_name: &'static str,
    pub static_text: Option<&'static str>,
    pub visible_rect: bool,
    pub clickable: bool,
    pub editable: bool,
    pub color: Color,
    pub size: Xy<Size>,
    pub position: Xy<Position>,
    pub container_mode: Option<ContainerMode>,
    pub z: f32,
}

impl Default for NodeParams {
    fn default() -> Self {
        Self {
            debug_name: "DEFAULT",
            static_text: None,
            clickable: false,
            visible_rect: false,
            color: Color::BLUE,
            size: Xy::new_symm(Size::PercentOfParent(0.5)),
            position: Xy::new_symm(Position::Start { padding: 5 }),
            container_mode: None,
            editable: false,
            z: 0.0,
        }
    }
}

impl NodeParams {
    pub const COLUMN: Self = Self {
        debug_name: "Column",
        static_text: None,
        clickable: true,
        visible_rect: false,
        color: Color {
            r: 0.0,
            g: 0.2,
            b: 0.7,
            a: 0.2,
        },
        size: Xy::new(Size::PercentOfParent(0.2), Size::PercentOfParent(1.0)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: Some(ContainerMode{
            main_axis_justify: Justify::Start,
            cross_axis_align: Align::Fill,
            main_axis: Axis::Y,
        }),
        editable: false,
        z: 0.0,
    };
    pub const ROW: Self = Self {
        debug_name: "Column",
        static_text: None,
        visible_rect: false,
        clickable: true,
        color: Color {
            r: 0.0,
            g: 0.2,
            b: 0.7,
            a: 0.2,
        },
        size: Xy::new(Size::PercentOfParent(1.0), Size::PercentOfParent(1.0)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: Some(ContainerMode{
            main_axis_justify: Justify::Start,
            cross_axis_align: Align::Fill,
            main_axis: Axis::X,
        }),
        editable: false,
        z: 0.0,
    };
    pub const FLOATING_WINDOW: Self = Self {
        debug_name: "FLOATING_WINDOW",
        static_text: None,
        clickable: true,
        visible_rect: false,
        color: Color {
            r: 0.7,
            g: 0.0,
            b: 0.0,
            a: 0.2,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.9)),
        position: Xy::new_symm(Position::Center),
        container_mode: None,
        editable: false,
        z: 0.0,
    };

    pub const BUTTON: Self = Self {
        debug_name: "Button",
        static_text: None,
        clickable: true,
        visible_rect: true,
        color: Color {
            r: 0.0,
            g: 0.1,
            b: 0.1,
            a: 0.9,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.17)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
        editable: false,
        z: 0.0,
    };

    pub const LABEL: Self = Self {
        debug_name: "label",
        static_text: None,
        clickable: false,
        visible_rect: true,
        color: Color {
            r: 0.0,
            g: 0.1,
            b: 0.1,
            a: 0.9,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.3)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
        editable: false,
        z: 0.0,
    };

    pub const TEXT_INPUT: Self = Self {
        debug_name: "label",
        static_text: None,
        clickable: true,
        visible_rect: true,
        color: Color {
            r: 0.0,
            g: 0.1,
            b: 0.1,
            a: 0.9,
        },
        size: Xy::new(Size::PercentOfParent(0.5), Size::PercentOfParent(0.1)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
        editable: true,
        z: 0.0,
    };
}

// NodeKey intentionally does not implement Clone, so that it's harder for the user to accidentally use duplicated Ids for different nodes.
// it's still too easy to clone an Id, but taking Clone out from that seems too annoying for now.
#[derive(Debug)]
pub struct NodeKey {
    // stuff like layout params, how it reacts to clicks, etc
    pub id: Id,
    pub params: NodeParams,
}

use std::hash::Hash;
impl NodeKey {
    pub const fn new(params: NodeParams, id: Id) -> Self {
        return Self { params, id };
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
            params: self.params.clone(),
        };
    }

    pub const fn with_size_x(mut self, size: f32) -> Self {
        self.params.size.0[0] = Size::PercentOfParent(size);
        return self;
    }

    pub const fn with_position_x(mut self, position: Position) -> Self {
        self.params.position.0[0] = position;
        return self;
    }

    pub const fn with_static_text(mut self, text: &'static str) -> Self {
        self.params.static_text = Some(text);
        return self;
    }

    pub const fn with_debug_name(mut self, text: &'static str) -> Self {
        self.params.debug_name = text;
        return self;
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.params.color = color;
        return self;
    }
}

#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
#[repr(C)]
// Layout has to match the one in the shader.
pub struct Rectangle {
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
        
        // if rect.last_frame_touched != self.current_frame {
            //     return (false, false);
            // }
            
        let mut mouse_pos = (
            self.mouse_pos.x / self.unifs.width,
            1.0 - (self.mouse_pos.y / self.unifs.height),
        );

        // transform mouse_pos into "opengl" centered coordinates
        mouse_pos.0 = (mouse_pos.0 * 2.0) - 1.0 ;
        mouse_pos.1 = (mouse_pos.1 * 2.0) - 1.0 ;

        let hovered = rect.x0 < mouse_pos.0
            && mouse_pos.0 < rect.x1
            && rect.y0 < mouse_pos.1
            && mouse_pos.1 < rect.y1;

        if hovered == false {
            return (false, false)
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
    Selection((GlyphonCursor, GlyphonCursor))
}

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
    pub nodes: FxHashMap<Id, Node>,

    pub parent_stack: Vec<Id>,


    pub part: PartialBorrowStuff,

    pub stack: Vec<Id>,

    pub last_tree_hash: u64,
    pub tree_hasher: FxHasher,

    pub clicked_stack: Vec<(Id, f32)>,
    pub hovered_stack: Vec<(Id, f32)>,
    pub clicked: Option<Id>,
    pub hovered: Option<Id>,

    pub focused: Option<Id>,

    // remember about animations (surely there will be)
    pub content_changed: bool,
    pub tree_changed: bool,

    pub t: f32,
}
impl Ui {
    pub fn update_text(&mut self, id: Id, text: impl ToString) {
        let text = text.to_string();
        let mut hasher = FxHasher::default();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // todo: return instead of unwrap. remove the others too...
        let text_id = self.nodes.get(&id).unwrap().text_id;
        let text_id = match text_id {
            Some(text_id) => {

                if hash == self.text_areas[text_id].last_hash {
                    // todo: I shouldn't have to do this, I don't think, it's visible as long as the node is visible?? 
                    self.text_areas[text_id].last_frame_touched = self.part.current_frame;
                    return;
                }
                self.text_areas[text_id].last_hash = hash;

                text_id
            },
            None => {
                let buffer = Buffer::new(&mut self.font_system, Metrics::new(42.0, 42.0));
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
                self.nodes.get_mut(&id).unwrap().text_id = text_id;
                text_id.unwrap()
            },
        };
        self.text_areas[text_id].buffer.set_text(
            &mut self.font_system,
            &text, 
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        self.text_areas[text_id].last_frame_touched = self.part.current_frame;

        self.content_changed = true;
    }

    pub fn update_color(&mut self, id: Id, color: Color) {
        if let Some(node) = self.nodes.get_mut(&id) {
            if node.params.color == color {
                return;
            } else {
                node.params.color = color;
                self.content_changed = true;
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
            width: config.width as f32,
            height: config.height as f32,
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
            nodes,
            gpu_vertex_buffer: vertex_buffer,
            uniform_buffer: resolution_buffer,
            bind_group,


            parent_stack,
            
            part: PartialBorrowStuff {
                mouse_pos: PhysicalPosition { x: 0., y: 0. },
                mouse_left_clicked: false,
                mouse_left_just_clicked: false,
                current_frame: 0,
                unifs: uniforms,
                t0: Instant::now(),
            },

            // stack for traversing
            stack: Vec::new(),

            clicked_stack: Vec::new(),
            clicked: None,
            hovered_stack: Vec::new(),
            hovered: None,
            focused: None,

            tree_hasher: FxHasher::default(),
            last_tree_hash: 0,
            content_changed: true,
            tree_changed: true,

            t: 0.0,
        }
    }

    pub fn column(&mut self, id: Id) {
        let key = NodeKey::new(NodeParams::COLUMN, id);
        self.add(key);
    }

    pub fn row(&mut self, id: Id) {
        let key = NodeKey::new(NodeParams::ROW, id);
        self.add(key);
    }

    pub fn floating_window(&mut self, id: Id) {
        let key = NodeKey::new(NodeParams::FLOATING_WINDOW, id);
        self.add(key);
    }

    // todo: deduplicate with refresh (maybe)
    pub fn add(&mut self, node_key: NodeKey) {
        let parent_id = *self.parent_stack.last().unwrap();

        let node_key_id = node_key.id;

        node_key_id.hash(&mut self.tree_hasher);

        let old_node = self.nodes.get_mut(&node_key_id);
        if old_node.is_none() {
            let mut text_id = None;
            if let Some(text) = node_key.params.static_text {
                // text size
                let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(42.0, 42.0));
                
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
            self.nodes.insert(node_key_id, new_node);
        } else {
            let old_node = old_node.unwrap();
            if let Some(text_id) = old_node.text_id {
                    self.text_areas[text_id].last_frame_touched = self.part.current_frame;
            }
            old_node.last_frame_touched = self.part.current_frame;
            old_node.children_ids.clear();
        }

        self.nodes
            .get_mut(&parent_id)
            .unwrap()
            .children_ids
            .push(node_key_id);
    }

    pub fn new_node(&self, node_key: NodeKey, parent_id: Id, text_id: Option<usize>) -> Node {
        Node {
            rect_id: None,
            rect: Xy::new_symm([0.0, 0.0]),
            text_id,
            parent_id,
            children_ids: Vec::new(),
            params: node_key.params,
            last_frame_touched: self.part.current_frame,
            last_frame_status: LastFrameStatus::Nothing,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) -> Option<()> {
        let id = self.focused?;
        let node = self.nodes.get(&id)?;
        let text_id = node.text_id?;
        println!(" {:#?}\n", event);
            
        if event.state.is_pressed() {
            let mut buffer = &mut self.text_areas[text_id].buffer;
            
            match &event.logical_key {
                winit::keyboard::Key::Named(named_key) => {
                    match named_key {
                        
                        NamedKey::ArrowLeft => {
                            match (self.key_modifiers.shift_key(), self.key_modifiers.control_key()) {
                                (true, true) => buffer.lines[0].text.control_shift_left_arrow(),
                                (true, false) => buffer.lines[0].text.shift_left_arrow(),
                                (false, true) => buffer.lines[0].text.control_left_arrow(),
                                (false, false) => buffer.lines[0].text.left_arrow(),
                            }
                        }
                        NamedKey::ArrowRight => {
                            match (self.key_modifiers.shift_key(), self.key_modifiers.control_key()) {
                                (true, true) => buffer.lines[0].text.control_shift_right_arrow(),
                                (true, false) => buffer.lines[0].text.shift_right_arrow(),
                                (false, true) => buffer.lines[0].text.control_right_arrow(),
                                (false, false) => buffer.lines[0].text.right_arrow(),
                            }
                        }
                        NamedKey::Backspace => {
                            if self.key_modifiers.control_key() {
                                buffer.lines[0].text.ctrl_backspace_unicode_word();
                            } else {
                                buffer.lines[0].text.backspace();
                            }
                            buffer.lines[0].reset();
                        }
                        NamedKey::Space => {
                            buffer.lines[0].text.insert_str_at_cursor(" ");
                            buffer.lines[0].reset();
                        },
                        _ => {},
                    }
                },
                winit::keyboard::Key::Character(new_char) => {
                    buffer.lines[0].text.insert_str_at_cursor(&new_char);
                    buffer.lines[0].reset();
                },
                winit::keyboard::Key::Unidentified(_) => {},
                winit::keyboard::Key::Dead(_) => {},
            };
            
            
        }
    
    
        return Some(())
    }

    pub fn handle_input_events(&mut self, full_event: &Event<()>) {
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
                    self.handle_keyboard_event(&event);
                }
                _ => {}
            }
        }
    }

    // todo: deduplicate the traversal with build_buffers, or just merge build_buffers inside here.
    // either way should wait to see how a real layout pass would look like
    // todo: layout has to be called BEFORE is_clicked and similar. maybe there's a way to force it or at least to print a warning?
    // or just do layout automatically somewhere? 
    pub fn layout(&mut self) {
        if ! self.needs_redraw() {
            return;
        }
        self.stack.clear();

        let mut parent_already_decided = false;
        let mut last_name = "root?";
        let mut last_rect = Xy::new_symm([0.0, 1.0]);
        let mut new_rect = last_rect;

        // push the direct children of the root without processing the root
        if let Some(root) = self.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter() {
                self.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.stack.pop() {
            // this mess over here is to avoid borrowing 2 nodes (which shouldn't even be a problem. but layout will change anyway.)
            let children_ids;
            let container;
            let debug_name;
            let len = Xy::new(new_rect[Axis::X][1] - new_rect[Axis::X][0], new_rect[Axis::Y][1] - new_rect[Axis::Y][0]);
            {            
                let current_node = self.nodes.get_mut(&current_node_id).unwrap();
                
                // println!("visiting {:?}, parent: {:?}", current_node.params.debug_name, last_name);
                // // println!("node {:?}", current_node.params.debug_name);
                // println!("rect {:?}", last_rect);
                // println!(" {:?}", "");

                children_ids = current_node.children_ids.clone();
                container = current_node.params.container_mode;
                debug_name = current_node.params.debug_name;


                
                if ! parent_already_decided {

                    for axis in [Axis::X, Axis::Y] {
                        match current_node.params.position[axis] {
                            Position::Start { padding } => {
                                let x0 = last_rect[axis][0] + (padding as f32 / self.part.unifs.width);
                                match current_node.params.size[axis] {
                                    Size::PercentOfParent(percent) => {
                                        let x1 = x0 + len[axis] * percent;
                                        new_rect[axis] = [x0, x1];
                                    },
                                }
                            },
                            Position::Center => {
                                let center = last_rect[axis][0] + len[axis] / 2.0;
                                match current_node.params.size[axis] {
                                    Size::PercentOfParent(percent) => {
                                        let width = len[axis] * percent;
                                        let x0 = center - width / 2.0;
                                        let x1 = center + width / 2.0;
                                        new_rect[axis] = [x0, x1];
                                    },
                                }
                            },
                        }
                    }
                    
                    current_node.rect = new_rect;
                }

                if let Some(id) = current_node.text_id {
                    self.text_areas[id].left = current_node.rect[X][0] * self.part.unifs.width;
                    self.text_areas[id].top = (1.0 - current_node.rect[Y][1]) * self.part.unifs.height;
                    self.text_areas[id].buffer.set_size(
                        &mut self.font_system,
                        100000.,
                        100000.,
                    );
                    self.text_areas[id]
                        .buffer
                        .shape_until_scroll(&mut self.font_system, false);

                }
            }

            match container {
                Some(mode) => {
                    // decide the children positions all at once
                    let padding = 5;
                    let mut main_0 = new_rect[mode.main_axis][0] + (padding as f32 / self.part.unifs.width);

                    for &child_id in children_ids.iter().rev() {
                        let child = self.nodes.get_mut(&child_id).unwrap();
                        let main_axis = mode.main_axis;
                        child.rect[main_axis][0] = main_0;

                        match child.params.size[main_axis] {
                            Size::PercentOfParent(percent) => {
                                let main_1 = main_0 + len[main_axis] * percent;
                                child.rect[main_axis][1] = main_1;
                                main_0 = main_1 + (padding as f32 / self.part.unifs.width);
                            },
                        }

                        let cross_axis = mode.main_axis.other();
                        match child.params.size[cross_axis] {
                            Size::PercentOfParent(percent) => {
                                let cross_0 = new_rect[cross_axis][0] + (padding as f32 / self.part.unifs.width);
                                let cross_1 = cross_0 + len[cross_axis] * percent;
                                child.rect[cross_axis][0] = cross_0;
                                child.rect[cross_axis][1] = cross_1;
                            },
                        }
                    }


                    for &child_id in children_ids.iter().rev() {
                        self.stack.push(child_id);
                        last_name = debug_name;
                        last_rect = new_rect;
                        parent_already_decided = true;
                    }

                },
                None => {
                    // just go to the children
                    for &child_id in children_ids.iter().rev() {
                        self.stack.push(child_id);
                        last_name = debug_name;
                        last_rect = new_rect;
                        parent_already_decided = false;
                    }
                },
            }

        }

        // println!("  ");

        // print_whole_tree
        // for (k, v) in &self.nodes {
        //     println!(" {:?}: {:#?}", k, v.params.debug_name);
        // }

        // println!("self.text_areas.len() {:?}", self.text_areas.len());
        // println!("self.rects.len() {:?}", self.rects.len());
    }

    pub fn is_clicked(&self, id: Id) -> bool {
        if !self.part.mouse_left_just_clicked {
            return false;
        }
        for (clicked_id, _z) in &self.clicked_stack {
            if *clicked_id == id {
                return true;
            }
        }
        return false;
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        self.part.unifs.width = size.width as f32;
        self.part.unifs.height = size.height as f32;
        self.content_changed = true;
        self.tree_changed = true;

        queue.write_buffer(&self.uniform_buffer, 0, &bytemuck::bytes_of(&self.part.unifs)[..16]);
    }

    pub fn update_gpu_time(&mut self, queue: &Queue) {
        // magical offset...
        queue.write_buffer(&self.uniform_buffer, 8, bytemuck::bytes_of(&self.t));
    }

    pub fn update_time(&mut self) {
        self.t = self.part.t0.elapsed().as_secs_f32();
    }

    pub fn build_buffers(&mut self) {
        if ! self.needs_redraw() {
            return;
        }

        self.rects.clear();
        self.stack.clear();

        // push the ui.direct children of the root without processing the root
        if let Some(root) = self.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter().rev() {
                self.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.stack.pop() {
            let current_node = self.nodes.get_mut(&current_node_id).unwrap();


            if current_node.params.visible_rect &&
            current_node.last_frame_touched == self.part.current_frame {
                // println!(" maybe too much?");
                // println!(" {:?}", current_node.params.debug_name);
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
        let focused_node = self.nodes.get(focused_id)?;
        let text_id = focused_node.text_id?;
        let focused_text_area = self.text_areas.get(text_id)?;
        
        match focused_text_area.buffer.lines[0].text.cursor() {
            StringCursor::Point(cursor) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];
                
                let (x, y) = cursor_pos_from_byte_offset(&focused_text_area.buffer, *cursor);
                
                let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                // we're counting on this always happening after layout. which should be safe.
                let x0 = ((x - 1.0) / self.part.unifs.width) * 2.0;
                let x1 = ((x + cursor_width) / self.part.unifs.width) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);
                
                let y0 = ((- y - cursor_height) / self.part.unifs.height) * 2.0;
                let y1 = ((- y ) / self.part.unifs.height) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);
              
                let cursor_rect = Rectangle {
                    x0,
                    x1,
                    y0,
                    y1,
                    r: 0.5,
                    g: 0.3,
                    b: 0.5,
                    a: 0.9,
                    last_hover: 0.0,
                    last_click: 0.0,
                    clickable: 0,
                    z: 0.0,
                    id: Id(0),
                    _padding: 0.0,
                    radius: 0.0,
                }; 

                self.rects.push(cursor_rect);
            },
            StringCursor::Selection(selection) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];
                
                let (x0, y0) = cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.start);
                let (x1, y1) = cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.end);
               

                let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                let x0 = ((x0 - 1.0) / self.part.unifs.width) * 2.0;
                let x1 = ((x1 + 1.0) / self.part.unifs.width) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);
                
                let y0 = ((- y0 - cursor_height) / self.part.unifs.height) * 2.0;
                let y1 = ((- y1 ) / self.part.unifs.height) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);

                let cursor_rect = Rectangle {
                    x0,
                    x1,
                    y0,
                    y1,
                    r: 0.5,
                    g: 0.3,
                    b: 0.5,
                    a: 0.9,
                    last_hover: 0.0,
                    last_click: 0.0,
                    clickable: 0,
                    z: 0.0,
                    id: Id(0),
                    _padding: 0.0,
                    radius: 0.0,
                }; 

                self.rects.push(cursor_rect);
            },
        }
        
        

        return Some(());
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        if self.needs_redraw() {

            let n = self.rects.len() as u32;
            if n > 0 {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(n));
                render_pass.draw(0..6, 0..n);
            }
            
            self.text_renderer.render(&self.atlas, render_pass).unwrap();
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        if self.needs_redraw() {

            self.gpu_vertex_buffer.queue_write(&self.rects[..], queue);
            
            self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                GlyphonResolution {
                    width: self.part.unifs.width as u32,
                    height: self.part.unifs.height as u32,
                },
                &mut self.text_areas,
                &mut self.cache,
                self.part.current_frame,
            )
            .unwrap();        
        }
    }

    // todo: this can be called at the end of prepare() or render() instead of in main(), maybe.
    pub fn finish_frame(&mut self) {
        // the root isn't processed in the div! stuff because there's usually nothing to do with it (except this)
        self.nodes
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();
    
        self.content_changed = false;
        self.tree_changed = false;
        self.part.current_frame += 1;
        self.part.mouse_left_just_clicked = false;
        self.tree_hasher = FxHasher::default();
    }

    // todo: this can be called at the start of layout() (before the early return) instead that in main, if nothing changes.
    pub fn finish_tree(&mut self) {
        let hash = self.tree_hasher.finish();
        self.tree_changed = hash != self.last_tree_hash;
        self.last_tree_hash = hash;
    }

    pub fn needs_redraw(&self) -> bool {
        return self.tree_changed || self.content_changed; 
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
        // there may be exceptions.
        // also, nothing is really specifying their z right now.
        // if there are overlapping rects with the same z, it would be nice to use the inverse order of the rects, so that it's consistent with what gets drawn on top, but I don't know how to do that right now -- (actually using rev() does basically the same thing).
        // one way would be to stick a lot of duplicated data in the rect (or a soa thing) and decide clicked over rects instead of nodes.
        // that might actually be good for speed as well. 
        // but it might be bad for order of operations.

        // reset old color mods

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
            let node = self.nodes.get_mut(&id).unwrap();
            node.last_hover = self.t;
        }

        let mut focused_anything = false;
        if let Some(id) = self.clicked {
            let node = self.nodes.get_mut(&id).unwrap();
            node.last_click = self.t;
            
            if node.params.editable {
                self.focused = self.clicked;
                focused_anything = true;
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.text_areas[id];
                let (x, y) = (
                    
                    self.part.mouse_pos.x - text_area.left,
                    self.part.mouse_pos.y - text_area.top
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
}


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
    PercentOfParent(f32),
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
    Start { padding: u32 },
    // End { padding: u32 },
    // TrustParent,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContainerMode {
    main_axis_justify: Justify,
    cross_axis_align: Align,
    main_axis: Axis,
}

#[derive(Debug, Clone, Copy)]
pub enum Justify {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Start,
    End,
    Center,
    Fill,
}

pub const NODE_ROOT_KEY: NodeKey = NodeKey {
    id: NODE_ROOT_ID,
    params: NODE_ROOT_PARAMS,
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
    size: Xy::new_symm(Size::PercentOfParent(1.0)),
    position: Xy::new_symm(Position::Start { padding: 0 }),
    container_mode: None,
    editable: false,
    z: 0.0,
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

#[repr(C)]
#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
pub struct Uniforms {
    pub width: f32,
    pub height: f32,
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

// this is a macro only for symmetry. probably not worth it over just ui.add(node_key).
#[macro_export]
macro_rules! add {
    ($ui:expr, $node_key:expr) => {
        $ui.add($node_key);
    };
    ($ui:expr, $node_key:expr, $code:block) => {
        $ui.add($node_key);

        $ui.parent_stack.push($node_key.id());
        $code;
        $ui.parent_stack.pop();
    };
}

// these have to be macros only because of the deferred pop().
#[macro_export]
macro_rules! column {
    ($ui:expr, $code:block) => {
        let anonymous_id = new_id!();
        $ui.column(anonymous_id);

        $ui.parent_stack.push(anonymous_id);
        $code;
        $ui.parent_stack.pop();
    };
}

#[macro_export]
macro_rules! row {
    ($ui:expr, $code:block) => {
        let anonymous_id = new_id!();
        $ui.row(anonymous_id);

        $ui.parent_stack.push(anonymous_id);
        $code;
        $ui.parent_stack.pop();
    };
}

#[macro_export]
macro_rules! floating_window {
    ($ui:expr, $code:block) => {
        let anonymous_id = new_id!();
        $ui.floating_window(anonymous_id);

        $ui.parent_stack.push(anonymous_id);
        $code;
        $ui.parent_stack.pop();
    };
}

pub trait Component {
    fn add(&mut self);
    // fn update(&mut self);
    // fn auto_interact(&mut self);
}

pub fn is_word_separator(c: char) -> bool {
    if c.is_alphanumeric() {
        return false;
    }
    // if c.is_whitespace() {
    //     return true;
    // }
    return true;
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


