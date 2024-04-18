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

use std::{collections::HashMap, marker::PhantomData, mem, sync::Arc};
const NODE_ROOT_ID: u64 = 0;

#[rustfmt::skip]
fn main() {
    let (event_loop, mut state) = init();

    event_loop.run(
        move |event, target| {
            state.handle_event(&event, target);
        }
    ).unwrap();
}

fn init() -> (EventLoop<()>, State<'static>) {
    let (width, height) = (1200, 800);
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_title("BLUE")
            .build(&event_loop)
            .unwrap(),
    );
    let size = window.inner_size();
    // let scale_factor = window.scale_factor();

    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::default(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    let surface = instance.create_surface(window.clone()).unwrap();

    let swapchain_format = TextureFormat::Bgra8UnormSrgb;
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

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
        width: width as f32,
        height: height as f32,
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

    // Set up text renderer
    let font_system = FontSystem::new();
    let cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
    let text_renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    // let physical_width = (width as f64 * scale_factor) as f32;
    // let physical_height = (height as f64 * scale_factor) as f32;

    // let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));
    // buffer.set_size(&mut font_system, physical_width, physical_height);
    // buffer.set_text(&mut font_system, "Hello world! 👋この動画の元になった作品ヽ༼ ຈل͜ຈ༽ ﾉヽ༼ ຈل͜ຈ༽ ﾉ\nヽ༼ ຈل͜ຈ༽\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    // buffer.shape_until_scroll(&mut font_system);

    // let text_areas = vec![TextArea {
    //     buffer,
    //     left: 10.0,
    //     top: 10.0,
    //     scale: 1.0,
    //     bounds: TextBounds {
    //         left: 0,
    //         top: 0,
    //         right: 900,
    //         bottom: 660,
    //     },
    //     default_color: GlyphonColor::rgb(255, 255, 255),
    //     depth: 0.0,
    // }];
    let text_areas = Vec::new();

    let mut nodes = HashMap::with_capacity(20);

    nodes.insert(
        0,
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
    parent_stack.push(0);

    let state = State {
        window,
        surface,
        config,
        device,
        cache,
        queue,
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
        
        node_tree_touched: false,
        
        // app state
        count: 0,
        counter_mode: true,
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: Arc<Window>,

    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
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
    pub nodes: HashMap<u64, Node>,

    pub parent_stack: Vec<u64>,

    pub count: i32,
    pub current_frame: u64,

    pub mouse_pos: PhysicalPosition<f32>,
    pub mouse_left_clicked: bool,
    pub mouse_left_just_clicked: bool,

    pub counter_mode: bool,
    pub node_tree_touched: bool,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,

    pub last_frame_touched: u64,

    pub text_id: Option<u32>,
    pub parent_id: u64,
    // todo: maybe switch with that prev/next thing
    pub children_ids: Vec<u64>,
    pub key: NodeKey,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeKey {
    // stuff like layout params, how it reacts to clicks, etc
    pub id: u64,
    pub text: Option<&'static str>,
    pub clickable: bool,
    pub color: Color,
    pub layout_x: LayoutMode,
    pub layout_y: LayoutMode,
    pub is_update: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutMode {
    PercentOfParent { start: f32, end: f32 },
    Fixed { start: u32, len: u32 },
    ChildrenSum {},
}

pub const NODE_ROOT_KEY: NodeKey = NodeKey {
    id: 0,
    text: None,
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
};

pub const INCREASE_BUTTON: NodeKey = NodeKey {
    id: 111111111,
    text: Some(&"Increase"),
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.1,
        b: 0.1,
        a: 0.9,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 100,
        len: 100,
    },
    is_update: false,
};

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey {
    id: 2222222,
    text: Some(&"Show counter"),
    clickable: true,
    color: Color {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 400,
        len: 100,
    },
    is_update: false,
};

pub const COUNT_LABEL: NodeKey = NodeKey {
    id: 555555,
    text: Some(&"0"),
    clickable: false,
    color: Color {
        r: 0.1,
        g: 0.3,
        b: 0.9,
        a: 0.6,
    },
    layout_x: LayoutMode::Fixed {
        start: 400,
        len: 100,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.3,
        end: 0.7,
    },
    is_update: false,
};

pub const COLUMN_1: NodeKey = NodeKey {
    id: 3333333,
    text: None,
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.2,
        b: 0.7,
        a: 0.2,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.8,
        end: 0.9,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.0,
        end: 1.0,
    },
    is_update: false,
};

pub const fn floating_window_1() -> NodeKey {
    return NodeKey {
        id: 77777777,
        text: None,
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
    };
}
impl NodeKey {
    pub const fn with_id(mut self, id: u64) -> Self {
        self.id = id;
        return self;
    }
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self.is_update = true;
        return self;
    }
}

pub const FLOATING_WINDOW_1: NodeKey = floating_window_1().with_id(34);

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::RedrawRequested => {
                    self.update();

                    self.window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
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

    pub fn update(&mut self) {
        self.nodes
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();

        self.node_tree_touched = false;

        div!((self, FLOATING_WINDOW_1) {

            // div!(self, COUNT_LABEL.with_text(&self.count.to_string()));

            div!((self, COLUMN_1) {

                div!(self, SHOW_COUNTER_BUTTON);

                if self.counter_mode {
                    let color = Color { r: 0.1 * (self.count as f32), g: 0.0, b: 0.0, a: 1.0 };
                    div!(self, INCREASE_BUTTON.with_color(color));
                }

            });

        });

        self.layout();
        // self.resolve_input();

        if self.is_clicked(INCREASE_BUTTON) {
            self.count += 1;
            println!("{:?}", self.count);
        }

        if self.is_clicked(SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
        }

        self.build_buffers();
        self.render();

        self.current_frame += 1;
        self.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.gpu_vertex_buffer
            .queue_write(&self.rects[..], &self.queue);

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                GlyphonResolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                &mut self.text_areas,
                &mut self.cache,
                self.current_frame,
            )
            .unwrap();

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            const GREY: wgpu::Color = wgpu::Color {
                r: 0.009,
                g: 0.017,
                b: 0.077,
                a: 1.0,
            };
            let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(GREY),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let n = self.rects.len() as u32;
            if n > 0 {
                r_pass.set_pipeline(&self.render_pipeline);
                r_pass.set_bind_group(0, &self.bind_group, &[]);
                r_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(n));
                r_pass.draw(0..6, 0..n);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer.render(&self.atlas, &mut pass).unwrap();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.atlas.trim();
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        let resolution = Resolution {
            width: size.width as f32,
            height: size.height as f32,
        };
        self.queue
            .write_buffer(&self.resolution_buffer, 0, bytemuck::bytes_of(&resolution));

        self.window.request_redraw();
    }

    pub fn build_buffers(&mut self) {
        self.rects.clear();
        let mut stack = Vec::<u64>::new();

        // push the direct children of the root without processing the root
        if let Some(root) = self.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter().rev() {
                stack.push(child_id);
            }
        }

        while let Some(current_node_id) = stack.pop() {
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
                stack.push(child_id);
            }
        }
    }

    // todo: deduplicate the layout with build_buffers
    pub fn layout(&mut self) {
        if self.node_tree_touched == false {
            println!("skipping layout");
            return;
        }
        
        let mut stack = Vec::<u64>::new();

        let mut last_rect_xs = (0.0, 1.0);
        let mut last_rect_ys = (0.0, 1.0);
        // push the direct children of the root without processing the root
        if let Some(root) = self.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter() {
                stack.push(child_id);
            }
        }

        while let Some(current_node_id) = stack.pop() {
            let current_node = self.nodes.get_mut(&current_node_id).unwrap();
            match current_node.key.layout_x {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = last_rect_xs.1 - last_rect_xs.0;
                    let x0 = last_rect_xs.0;
                    last_rect_xs = (x0 + len * start, x0 + len * end)
                }
                LayoutMode::ChildrenSum {} => todo!(),
                LayoutMode::Fixed { start, len } => {
                    let x0 = last_rect_xs.0;
                    last_rect_xs = (
                        x0 + (start as f32) / (self.config.width as f32),
                        x0 + ((start + len) as f32) / (self.config.width as f32),
                    )
                }
            }
            match current_node.key.layout_y {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = last_rect_ys.1 - last_rect_ys.0;
                    let y0 = last_rect_ys.0;
                    last_rect_ys = (y0 + len * start, y0 + len * end)
                }
                LayoutMode::Fixed { start, len } => {
                    let y0 = last_rect_ys.0;
                    last_rect_ys = (
                        y0 + (start as f32) / (self.config.height as f32),
                        y0 + ((start + len) as f32) / (self.config.height as f32),
                    )
                }
                LayoutMode::ChildrenSum {} => todo!(),
            }

            current_node.x0 = last_rect_xs.0;
            current_node.x1 = last_rect_xs.1;
            current_node.y0 = last_rect_ys.0;
            current_node.y1 = last_rect_ys.1;

            if let Some(id) = current_node.text_id {
                self.text_areas[id as usize].left = current_node.x0 * (self.config.width as f32);
                self.text_areas[id as usize].top = (1.0 - current_node.y1) * (self.config.height as f32);
                self.text_areas[id as usize].buffer.set_size(&mut self.font_system, 100000., 100000.,);
                self.text_areas[id as usize].buffer.shape_until_scroll(&mut self.font_system);
            }

            // do I really need iter.rev() here? why?
            for &child_id in current_node.children_ids.iter().rev() {
                stack.push(child_id);
            }
        }

        // // print_whole_tree
        // for (k, v) in &self.nodes {
            // println!(" {:?}: {:#?}", k, v);
        // }

        // println!("self.text_areas.len() {:?}", self.text_areas.len());
        // println!("self.rects.len() {:?}", self.rects.len());
    }

    pub fn div(&mut self, node_key: NodeKey) {
        let parent_id = self.parent_stack.last().unwrap().clone();

        let old_node = self.nodes.get_mut(&node_key.id);
        if old_node.is_none() || node_key.is_update {
            self.node_tree_touched = true; 

            let text_id = match node_key.text {
                Some(text) => {

                    let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(30.0, 42.0));
                    buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), Shaping::Advanced);
                
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
                    Some((self.text_areas.len() - 1) as u32)
                },
                None => None,
            };

            let new_node = self.new_node(node_key, parent_id, text_id);
            self.nodes.insert(node_key.id, new_node);

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
            .push(node_key.id);
    }

    pub fn new_node(&self, node_key: NodeKey, parent_id: u64, text_id: Option<u32>) -> Node {
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

    // in the future, do the full tree pass (for covered stuff etc)
    pub fn is_clicked(&self, button: NodeKey) -> bool {
        if !self.mouse_left_just_clicked {
            return false;
        }

        let node = self.nodes.get(&button.id);
        if let Some(node) = node {
            let mouse_pos = (
                self.mouse_pos.x / (self.config.width as f32),
                1.0 - (self.mouse_pos.y / (self.config.height as f32)),
            );
            if node.x0 < mouse_pos.0 && mouse_pos.0 < node.x1 {
                if node.y0 < mouse_pos.1 && mouse_pos.1 < node.y1 {
                    return true;
                }
            }
        }

        return false;
    }
}

// these have to be macros only because of the deferred pop().
// todo: pass "ui" or something instead of self.

#[macro_export]
macro_rules! div {
    // non-leaf, has to manage the stack and pop() after the code
    (($self:ident, $node_key:expr) $code:block) => {
        $self.div($node_key);

        $self.parent_stack.push($node_key.id);
        $code;
        $self.parent_stack.pop();
    };
    // leaf. doesn't need to touch the stack. doesn't actually need to be a macro except for symmetry.
    ($self:ident, $node_key:expr) => {
        $self.div($node_key);
    };
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

#[repr(C)]
#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
pub struct Resolution {
    width: f32,
    height: f32,
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
