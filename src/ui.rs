use glyphon::Resolution as GlyphonResolution;
use std::{collections::HashMap, marker::PhantomData, mem};

use crate::{id, HEIGHT, SWAPCHAIN_FORMAT, WIDTH};

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Id(pub u64);
pub const NODE_ROOT_ID: Id = Id(0);

#[derive(Debug, Clone)]
pub struct NodeKey {
    // stuff like layout params, how it reacts to clicks, etc
    pub id: Id,
    pub static_text: Option<&'static str>,
    pub dyn_text: Option<String>,
    pub clickable: bool,
    pub color: Color,
    pub layout_x: LayoutMode,
    pub layout_y: LayoutMode,
    pub is_update: bool,
    pub is_layout_update: bool,
}
impl NodeKey {
    pub const fn button() -> NodeKey {
        return NodeKey {
            id: id!(),
            static_text: None,
            dyn_text: None,
            clickable: true,
            color: Color {
                r: 0.0,
                g: 0.1,
                b: 0.1,
                a: 0.9,
            },
            layout_x: LayoutMode::Fixed {
                start: 100,
                len: 100,
            },
            layout_y: LayoutMode::Fixed {
                start: 100,
                len: 100,
            },
            is_update: false,
            is_layout_update: false,
        };
    }
    pub const fn label() -> NodeKey {
        return NodeKey {
            id: id!(),
            static_text: None,
            dyn_text: None,
            clickable: true,
            color: Color {
                r: 0.0,
                g: 0.1,
                b: 0.1,
                a: 0.9,
            },
            layout_x: LayoutMode::Fixed {
                start: 100,
                len: 100,
            },
            layout_y: LayoutMode::Fixed {
                start: 100,
                len: 100,
            },
            is_update: false,
            is_layout_update: false,
        };
    }

    pub const fn with_id(mut self, id: Id) -> Self {
        self.id = id;
        return self;
    }
    pub const fn with_static_text(mut self, text: &'static str) -> Self {
        self.static_text = Some(text);
        self.is_update = true;
        return self;
    }
    pub const fn with_layout_x(mut self, layout: LayoutMode) -> Self {
        self.layout_x = layout;
        self.is_update = true;
        return self;
    }
    pub const fn with_layout_y(mut self, layout: LayoutMode) -> Self {
        self.layout_y = layout;
        self.is_update = true;
        return self;
    }
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self.is_update = true;
        return self;
    }
    pub fn with_text(mut self, text: impl ToString) -> Self {
        // todo: could keep a hash of the last to_string value and compare it, so you could skip an allocation if it's the same.
        // it's pretty cringe to allocate the string every frame for no reason.
        self.dyn_text = Some(text.to_string());
        self.is_update = true;
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
    pub nodes: HashMap<Id, Node>,

    pub parent_stack: Vec<Id>,

    pub current_frame: u64,

    pub mouse_pos: PhysicalPosition<f32>,
    pub mouse_left_clicked: bool,
    pub mouse_left_just_clicked: bool,
    pub stack: Vec<Id>,
}
impl Ui {
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
            width: WIDTH as f32,
            height: HEIGHT as f32,
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
        let mut atlas = TextAtlas::new(device, queue, SWAPCHAIN_FORMAT);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let text_areas = Vec::new();

        let mut nodes = HashMap::with_capacity(20);

        nodes.insert(
            NODE_ROOT_ID,
            Node {
                x0: -1.0,
                x1: 1.0,
                y0: -1.0,
                y1: 1.0,
                text_id: None,
                parent_id: NODE_ROOT_ID,
                children_ids: Vec::new(),
                key: NODE_ROOT_KEY,
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
                width: WIDTH as f32,
                height: HEIGHT as f32,
            },
            parent_stack,
            current_frame: 0,

            mouse_pos: PhysicalPosition { x: 0., y: 0. },
            mouse_left_clicked: false,
            mouse_left_just_clicked: false,

            // stack for traversing
            stack: Vec::new(),
        }
    }

    pub fn column(&mut self, id: Id) {
        let key = NodeKey {
            id,
            static_text: None,
            dyn_text: None,
            clickable: true,
            color: Color {
                r: 0.0,
                g: 0.2,
                b: 0.7,
                a: 0.2,
            },
            layout_x: LayoutMode::PercentOfParent {
                start: 0.7,
                end: 0.9,
            },
            layout_y: LayoutMode::PercentOfParent {
                start: 0.0,
                end: 1.0,
            },
            is_update: false,
            is_layout_update: false,
        };
        self.div(key);
    }


    pub fn floating_window(&mut self, id: Id) {
        let key = NodeKey {
            id,
            static_text: None,
            dyn_text: None,
            clickable: true,
            color: Color {
                r: 0.7,
                g: 0.0,
                b: 0.0,
                a: 0.2,
            },
            layout_x: LayoutMode::PercentOfParent {
                start: 0.1,
                end: 0.9,
            },
            layout_y: LayoutMode::PercentOfParent {
                start: 0.1,
                end: 0.9,
            },
            is_update: false,
            is_layout_update: false,
        };
        self.div(key);
    }


    pub fn div(&mut self, node_key: NodeKey) {
        let parent_id = *self.parent_stack.last().unwrap();

        let node_key_id = node_key.id;
        let old_node = self.nodes.get_mut(&node_key_id);
        if old_node.is_none() {
            let has_text = node_key.static_text.is_some() || node_key.dyn_text.is_some();
            let mut text_id = None;
            if has_text {
                let mut text: &str = "Remove this";

                if let Some(ref dyn_text) = node_key.dyn_text {
                    text = &dyn_text;
                } else if let Some(static_text) = node_key.static_text {
                    text = static_text;
                }

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
        } else if node_key.is_update || node_key.is_layout_update {
            // instead of reinserting, could just handle all update possibilities by his own.
            let old_node = old_node.unwrap();
            if let Some(text_id) = old_node.text_id {
                let mut text: &str = "Remove this";

                if let Some(ref dyn_text) = node_key.dyn_text {
                    text = &dyn_text;
                } else if let Some(static_text) = node_key.static_text {
                    text = static_text;
                }

                self.text_areas[text_id as usize].buffer.set_text(
                    &mut self.font_system,
                    text,
                    Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );
                self.text_areas[text_id as usize].last_frame_touched = self.current_frame;
            }
            let text_id = old_node.text_id;
            let new_node = self.new_node(node_key, parent_id, text_id);

            self.nodes.insert(node_key_id, new_node);
        } else {
            let old_node = old_node.unwrap();
            old_node.children_ids.clear();
            old_node.last_frame_touched = self.current_frame;
            if let Some(text_id) = old_node.text_id {
                self.text_areas[text_id as usize].last_frame_touched = self.current_frame;
            }
        }

        self.nodes
            .get_mut(&parent_id)
            .unwrap()
            .children_ids
            .push(node_key_id);
    }

    pub fn new_node(&self, node_key: NodeKey, parent_id: Id, text_id: Option<u32>) -> Node {
        Node {
            x0: 0.0,
            x1: 1.0,
            y0: 0.0,
            y1: 1.0,
            text_id,
            parent_id,
            children_ids: Vec::new(),
            key: node_key,
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
    pub fn layout(&mut self) {
        self.stack.clear();

        let mut last_rect_xs = (0.0, 1.0);
        let mut last_rect_ys = (0.0, 1.0);

        // push the direct children of the root without processing the root
        if let Some(root) = self.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter() {
                self.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.stack.pop() {
            let current_node = self.nodes.get_mut(&current_node_id).unwrap();

            let mut new_rect_xs = last_rect_xs;
            let mut new_rect_ys = last_rect_ys;

            match current_node.key.layout_x {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = new_rect_xs.1 - new_rect_xs.0;
                    let x0 = new_rect_xs.0;
                    new_rect_xs = (x0 + len * start, x0 + len * end)
                }
                LayoutMode::ChildrenSum {} => todo!(),
                LayoutMode::Fixed { start, len } => {
                    let x0 = new_rect_xs.0;
                    new_rect_xs = (
                        x0 + (start as f32) / self.resolution.width,
                        x0 + ((start + len) as f32) / self.resolution.width,
                    )
                }
            }
            match current_node.key.layout_y {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = new_rect_ys.1 - new_rect_ys.0;
                    let y0 = new_rect_ys.0;
                    new_rect_ys = (y0 + len * start, y0 + len * end)
                }
                LayoutMode::Fixed { start, len } => {
                    let y0 = new_rect_ys.0;
                    new_rect_ys = (
                        y0 + (start as f32) / self.resolution.height,
                        y0 + ((start + len) as f32) / self.resolution.height,
                    )
                }
                LayoutMode::ChildrenSum {} => todo!(),
            }

            current_node.x0 = new_rect_xs.0;
            current_node.x1 = new_rect_xs.1;
            current_node.y0 = new_rect_ys.0;
            current_node.y1 = new_rect_ys.1;

            if let Some(id) = current_node.text_id {
                self.text_areas[id as usize].left = current_node.x0 * self.resolution.width;
                self.text_areas[id as usize].top = (1.0 - current_node.y1) * self.resolution.height;
                self.text_areas[id as usize].buffer.set_size(
                    &mut self.font_system,
                    100000.,
                    100000.,
                );
                self.text_areas[id as usize]
                    .buffer
                    .shape_until_scroll(&mut self.font_system);
            }

            // do I really need iter.rev() here? why?
            for &child_id in current_node.children_ids.iter().rev() {
                self.stack.push(child_id);

                last_rect_xs.0 = new_rect_xs.0;
                last_rect_xs.1 = new_rect_xs.1;
                last_rect_ys.0 = new_rect_ys.0;
                last_rect_ys.1 = new_rect_ys.1;
            }
        }

        // println!(" {:?}", "  ");

        // print_whole_tree
        // for (k, v) in &self.nodes {
        //     println!(" {:?}: {:#?}", k, v.key.id);
        // }

        // println!("self.text_areas.len() {:?}", self.text_areas.len());
        // println!("self.rects.len() {:?}", self.rects.len());
    }

    // in the future, do the full tree pass (for covered stuff etc)
    pub fn is_clicked(&self, button: NodeKey) -> bool {
        if !self.mouse_left_just_clicked {
            return false;
        }

        let node = self.nodes.get(&button.id);
        if let Some(node) = node {
            if node.last_frame_touched != self.current_frame {
                return false;
            }

            let mouse_pos = (
                self.mouse_pos.x / self.resolution.width,
                1.0 - (self.mouse_pos.y / self.resolution.height),
            );
            if node.x0 < mouse_pos.0
                && mouse_pos.0 < node.x1
                && node.y0 < mouse_pos.1
                && mouse_pos.1 < node.y1
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

            if current_node.last_frame_touched == self.current_frame {
                self.rects.push(Rectangle {
                    x0: current_node.x0 * 2. - 1.,
                    x1: current_node.x1 * 2. - 1.,
                    y0: current_node.y0 * 2. - 1.,
                    y1: current_node.y1 * 2. - 1.,
                    r: current_node.key.color.r,
                    g: current_node.key.color.g,
                    b: current_node.key.color.b,
                    a: current_node.key.color.a,
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
            )
            .unwrap();

        // self.ui.atlas.trim();
        self.nodes
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,

    pub last_frame_touched: u64,

    pub text_id: Option<u32>,
    pub parent_id: Id,
    // todo: maybe switch with that prev/next thing
    pub children_ids: Vec<Id>,
    pub key: NodeKey,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutMode {
    PercentOfParent { start: f32, end: f32 },
    Fixed { start: u32, len: u32 },
    ChildrenSum {},
}

pub const NODE_ROOT_KEY: NodeKey = NodeKey {
    id: NODE_ROOT_ID,
    static_text: None,
    dyn_text: None,
    clickable: false,
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.0,
        end: 1.0,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.0,
        end: 1.0,
    },
    is_update: false,
    is_layout_update: false,
};

#[macro_export]
macro_rules! id {
    () => {{
        // todo: this is trash, I think.
        $crate::ui::Id((std::line!() as u64) << 32 | (std::column!() as u64))
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

// these have to be macros only because of the deferred pop().
// todo: pass "ui" or something instead of self.

#[macro_export]
macro_rules! div {
    // non-leaf, has to manage the stack and pop() after the code
    (($ui:expr, $node_key:expr) $code:block) => {
        $ui.div($node_key);

        $ui.parent_stack.push($node_key.id);
        $code;
        $ui.parent_stack.pop();
    };
    // leaf. doesn't need to touch the stack. doesn't actually need to be a macro except for symmetry.
    ($ui:expr, $node_key:expr) => {
        $ui.div($node_key);
    };
}

#[macro_export]
macro_rules! column {
    (($ui:expr) $code:block) => {
        let anonymous_id = id!();
        $ui.column(anonymous_id);

        $ui.parent_stack.push(anonymous_id);
        $code;
        $ui.parent_stack.pop();
    };
}

#[macro_export]
macro_rules! floating_window {
    (($ui:expr) $code:block) => {
        let anonymous_id = id!();
        $ui.floating_window(anonymous_id);

        $ui.parent_stack.push(anonymous_id);
        $code;
        $ui.parent_stack.pop();
    };
}