use std::{marker::PhantomData, mem, collections::HashMap};

use crate::{id, WIDTH, HEIGHT, SWAPCHAIN_FORMAT};

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution as GlyphonResolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer,
};
use wgpu::{
    util::{self, DeviceExt},
    vertex_attr_array, BindGroup, BufferAddress, BufferUsages, ColorTargetState,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RequestAdapterOptions,
    Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
    VertexAttribute, VertexBufferLayout, VertexStepMode,
};
use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Id(pub u64);
pub const NODE_ROOT_ID: Id = Id(0);


pub const fn floating_window_1() -> NodeKey {
    return NodeKey {
        id: id!(),
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
}
impl NodeKey {
    pub const fn with_id(mut self, id: Id) -> Self {
        self.id = id;
        return self;
    }
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self.is_update = true;
        return self;
    }
    pub fn with_static_text(mut self, text: &'static str) -> Self {
        self.static_text = Some(text);
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
        let mut atlas = TextAtlas::new(&device, &queue, SWAPCHAIN_FORMAT);
        let text_renderer =
            TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);

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

            parent_stack,
            current_frame: 0,

            mouse_pos: PhysicalPosition { x: 0., y: 0. },
            mouse_left_clicked: false,
            mouse_left_just_clicked: false,

            // stack for traversing
            stack: Vec::new(),
        }
    }

    pub fn div(&mut self, node_key: NodeKey) {
        let parent_id = self.parent_stack.last().unwrap().clone();

        let node_key_id = node_key.id;
        let old_node = self.nodes.get_mut(&node_key_id);
        if old_node.is_none() {
            let has_text = node_key.static_text.is_some() || node_key.dyn_text.is_some();
            let mut text_id = None;
            if has_text {
                let mut text: &str = &"Remove this";

                if let Some(ref dyn_text) = node_key.dyn_text {
                    text = &dyn_text;
                } else {
                    if let Some(static_text) = node_key.static_text {
                        text = static_text;
                    }
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
            self.nodes.insert(node_key_id.clone(), new_node);
        } else if node_key.is_update || node_key.is_layout_update {
            // instead of reinserting, could just handle all update possibilities by his own.
            let old_node = old_node.unwrap();
            if let Some(text_id) = old_node.text_id {
                let mut text: &str = &"Remove this";

                if let Some(ref dyn_text) = node_key.dyn_text {
                    text = &dyn_text;
                } else {
                    if let Some(static_text) = node_key.static_text {
                        text = static_text;
                    }
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

            self.nodes.insert(node_key_id.clone(), new_node);
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
        crate::ui::Id((std::line!() as u64) << 32 | (std::column!() as u64))
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
