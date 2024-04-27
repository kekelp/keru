use glyphon::Resolution as GlyphonResolution;
use rustc_hash::{FxHashMap, FxHasher};

use std::{hash::Hasher, marker::PhantomData, mem, ops::{Index, IndexMut}};

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
    event::{ElementState, Event, MouseButton, WindowEvent},
};

use Axis::{X, Y};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Id(pub(crate) u64);

pub const NODE_ROOT_ID: Id = Id(0);

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
    pub clickable: bool,
    pub color: Color,
    pub size: Xy<Size>,
    pub position: Xy<Position>,
    pub container_mode: Option<ContainerMode>,
}

impl Default for NodeParams {
    fn default() -> Self {
        Self {
            debug_name: "DEFAULT",
            static_text: None,
            clickable: false,
            color: Color::BLUE,
            size: Xy::new_symm(Size::PercentOfParent(0.5)),
            position: Xy::new_symm(Position::Start { padding: 5 }),
            container_mode: None,
        }
    }
}

impl NodeParams {
    pub const COLUMN: Self = Self {
        debug_name: "Column",
        static_text: None,
        clickable: true,
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
    };
    pub const ROW: Self = Self {
        debug_name: "Column",
        static_text: None,
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
    };
    pub const FLOATING_WINDOW: Self = Self {
        debug_name: "FLOATING_WINDOW",
        static_text: None,
        clickable: true,
        color: Color {
            r: 0.7,
            g: 0.0,
            b: 0.0,
            a: 0.2,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.9)),
        position: Xy::new_symm(Position::Center),
        container_mode: None,
    };

    pub const BUTTON: Self = Self {
        debug_name: "Button",
        static_text: None,
        clickable: true,
        color: Color {
            r: 0.0,
            g: 0.1,
            b: 0.1,
            a: 0.9,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.17)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
    };

    pub const LABEL: Self = Self {
        debug_name: "label",
        static_text: None,
        clickable: true,
        color: Color {
            r: 0.0,
            g: 0.1,
            b: 0.1,
            a: 0.9,
        },
        size: Xy::new_symm(Size::PercentOfParent(0.3)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
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
}
impl Rectangle {
    pub fn buffer_desc() -> [VertexAttribute; 3] {
        return vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
        ];
    }
}

#[derive(Debug, Clone, Copy)]
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
}

pub struct Ui {
    pub gpu_vertex_buffer: TypedGpuBuffer<Rectangle>,
    pub render_pipeline: RenderPipeline,

    pub resolution: Resolution,
    pub resolution_buffer: wgpu::Buffer,
    pub bind_group: BindGroup,

    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,

    pub rects: Vec<Rectangle>,
    pub text_areas: Vec<TextArea>,
    pub nodes: FxHashMap<Id, Node>,

    pub parent_stack: Vec<Id>,

    pub current_frame: u64,

    pub mouse_pos: PhysicalPosition<f32>,
    pub mouse_left_clicked: bool,
    pub mouse_left_just_clicked: bool,
    pub stack: Vec<Id>,

    pub immediate_mode: bool,
}
impl Ui {
    // todo: check if the string is different and skip...?
    pub fn update_text(&mut self, id: Id, text: impl ToString) {
        let text_id = self.nodes.get(&id).unwrap().text_id;
        let text_id = match text_id {
            Some(text_id) => text_id,
            None => {
                let buffer = Buffer::new(&mut self.font_system, Metrics::new(30.0, 42.0));
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
                    last_frame_touched: self.current_frame,
                };

                self.text_areas.push(text_area);
                let text_id = Some((self.text_areas.len() - 1) as u32);
                self.nodes.get_mut(&id).unwrap().text_id = text_id;
                text_id.unwrap()
            },
        };
        self.text_areas[text_id as usize].buffer.set_text(
            &mut self.font_system,
            &text.to_string(), 
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        self.text_areas[text_id as usize].last_frame_touched = self.current_frame;
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

        let resolution = Resolution {
            width: config.width as f32,
            height: config.height as f32,
        };
        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Buffer"),
            contents: bytemuck::bytes_of(&resolution),
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
            primitive: wgpu::PrimitiveState::default(),
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

        nodes.insert(
            NODE_ROOT_ID,
            Node {
                rect: Xy::new_symm([-1.0, 1.0]),
                text_id: None,
                parent_id: NODE_ROOT_ID,
                children_ids: Vec::new(),
                params: NODE_ROOT_PARAMS,
                last_frame_touched: 0,
            },
        );

        let mut parent_stack = Vec::with_capacity(7);
        parent_stack.push(NODE_ROOT_ID);

        Self {
            cache,
            render_pipeline,
            atlas,
            text_renderer,
            font_system,
            text_areas,
            rects: Vec::with_capacity(20),
            nodes,
            gpu_vertex_buffer: vertex_buffer,
            resolution_buffer,
            bind_group,

            resolution: Resolution {
                width: config.width as f32,
                height: config.height as f32,
            },
            parent_stack,
            current_frame: 0,

            mouse_pos: PhysicalPosition { x: 0., y: 0. },
            mouse_left_clicked: false,
            mouse_left_just_clicked: false,

            // stack for traversing
            stack: Vec::new(),

            immediate_mode: true,
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
        let old_node = self.nodes.get_mut(&node_key_id);
        if old_node.is_none() {
            let mut text_id = None;
            if let Some(text) = node_key.params.static_text {
                let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(30.0, 42.0));
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
                    last_frame_touched: self.current_frame,
                };

                self.text_areas.push(text_area);
                text_id = Some((self.text_areas.len() - 1) as u32);
            }

            let new_node = self.new_node(node_key, parent_id, text_id);
            self.nodes.insert(node_key_id, new_node);
        } else {
            // instead of reinserting, could just handle all update possibilities by his own.
            let old_node = old_node.unwrap();
            if let Some(text_id) = old_node.text_id {
                if let Some(text) = node_key.params.static_text {
                        
                        self.text_areas[text_id as usize].buffer.set_text(
                            &mut self.font_system,
                        text,
                        Attrs::new().family(Family::SansSerif),
                        Shaping::Advanced,
                    );
                    self.text_areas[text_id as usize].last_frame_touched = self.current_frame;
                }
            }
            let text_id = old_node.text_id;
            let new_node = self.new_node(node_key, parent_id, text_id);

            self.nodes.insert(node_key_id, new_node);
        }

        self.nodes
            .get_mut(&parent_id)
            .unwrap()
            .children_ids
            .push(node_key_id);
    }

    pub fn new_node(&self, node_key: NodeKey, parent_id: Id, text_id: Option<u32>) -> Node {
        Node {
            rect: Xy::new_symm([0.0, 1.0]),
            text_id,
            parent_id,
            children_ids: Vec::new(),
            params: node_key.params,
            last_frame_touched: self.current_frame,
        }
    }

    pub fn handle_input_events(&mut self, event: &Event<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos.x = position.x as f32;
                    self.mouse_pos.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        if *state == ElementState::Pressed {
                            self.mouse_left_clicked = true;
                            if !self.mouse_left_just_clicked {
                                self.mouse_left_just_clicked = true;
                            }
                        } else {
                            self.mouse_left_clicked = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // todo: deduplicate the traversal with build_buffers, or just merge build_buffers inside here.
    // either way should wait to see how a real layout pass would look like
    // laying eggs
    pub fn layout(&mut self) {
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
            let children_ids;
            let container;
            let debug_name;
            let len = Xy::new(new_rect[Axis::X][1] - new_rect[Axis::X][0], new_rect[Axis::Y][1] - new_rect[Axis::Y][0]);
            {            
                let current_node = self.nodes.get_mut(&current_node_id).unwrap();
                children_ids = current_node.children_ids.clone();
                container = current_node.params.container_mode;
                debug_name = current_node.params.debug_name;

                // println!("visiting {:?}, parent: {:?}", current_node.params.debug_name, last_name);

                
                if ! parent_already_decided {

                    for axis in [Axis::X, Axis::Y] {
                        match current_node.params.position[axis] {
                            Position::Start { padding } => {
                                let x0 = last_rect[axis][0] + (padding as f32 / self.resolution.width);
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
                    self.text_areas[id as usize].left = current_node.rect[X][0] * self.resolution.width;
                    self.text_areas[id as usize].top = (1.0 - current_node.rect[Y][1]) * self.resolution.height;
                    self.text_areas[id as usize].buffer.set_size(
                        &mut self.font_system,
                        100000.,
                        100000.,
                    );
                    self.text_areas[id as usize]
                        .buffer
                        .shape_until_scroll(&mut self.font_system);

                }
            }

            match container {
                Some(mode) => {
                    // decide the children positions all at once
                    let padding = 5;
                    let mut main_0 = new_rect[mode.main_axis][0] + (padding as f32 / self.resolution.width);

                    for &child_id in children_ids.iter().rev() {
                        let child = self.nodes.get_mut(&child_id).unwrap();
                        let main_axis = mode.main_axis;
                        child.rect[main_axis][0] = main_0;

                        match child.params.size[main_axis] {
                            Size::PercentOfParent(percent) => {
                                let main_1 = main_0 + len[main_axis] * percent;
                                child.rect[main_axis][1] = main_1;
                                main_0 = main_1 + (padding as f32 / self.resolution.width);
                            },
                        }

                        let cross_axis = mode.main_axis.other();
                        match child.params.size[cross_axis] {
                            Size::PercentOfParent(percent) => {
                                let cross_0 = new_rect[cross_axis][0] + (padding as f32 / self.resolution.width);
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
        //     println!(" {:?}: {:#?}", k, v.key.id);
        // }

        // println!("self.text_areas.len() {:?}", self.text_areas.len());
        // println!("self.rects.len() {:?}", self.rects.len());
    }

    // in the future, do the full tree pass (for covered stuff etc)
    // probably better to take just the id (for performance)
    pub fn is_clicked(&self, button: NodeKey) -> bool {
        if !self.mouse_left_just_clicked {
            return false;
        }

        let node = self.nodes.get(&button.id);
        if let Some(node) = node {
            if self.immediate_mode && (node.last_frame_touched != self.current_frame) {
                return false;
            }

            let mouse_pos = (
                self.mouse_pos.x / self.resolution.width,
                1.0 - (self.mouse_pos.y / self.resolution.height),
            );
            if node.rect[X][0] < mouse_pos.0
                && mouse_pos.0 < node.rect[X][1]
                && node.rect[Y][0] < mouse_pos.1
                && mouse_pos.1 < node.rect[Y][1]
            {
                return true;
            }
        }

        return false;
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        let resolution = Resolution {
            width: size.width as f32,
            height: size.height as f32,
        };
        self.resolution = resolution;
        queue.write_buffer(&self.resolution_buffer, 0, bytemuck::bytes_of(&resolution));
    }

    pub fn build_buffers(&mut self) {
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

            if current_node.last_frame_touched == self.current_frame || self.immediate_mode == false
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
                });
            }

            for &child_id in current_node.children_ids.iter() {
                self.stack.push(child_id);
            }
        }
    }

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
                    width: self.resolution.width as u32,
                    height: self.resolution.height as u32,
                },
                &mut self.text_areas,
                &mut self.cache,
                self.current_frame,
                self.immediate_mode,
            )
            .unwrap();

        // self.ui.atlas.trim();

        // the root isn't processed in the div! stuff because there's usually nothing to do with it (except this)
        if self.immediate_mode {
            self.nodes
                .get_mut(&NODE_ROOT_ID)
                .unwrap()
                .children_ids
                .clear();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub rect: Xy<[f32; 2]>,
    // pub x0: f32,
    // pub x1: f32,
    // pub y0: f32,
    // pub y1: f32,

    pub last_frame_touched: u64,

    pub text_id: Option<u32>,
    pub parent_id: Id,
    // todo: maybe switch with that prev/next thing
    pub children_ids: Vec<Id>,
    pub params: NodeParams,
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
pub struct Resolution {
    pub width: f32,
    pub height: f32,
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
