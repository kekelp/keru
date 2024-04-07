use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution as GlyphonResolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute, vertex_attr_array, util::{DeviceExt, self}, BufferUsages, VertexBufferLayout, VertexStepMode, BufferAddress, RenderPipeline, BindGroup,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use std::{sync::Arc, mem, marker::PhantomData};

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
            .with_title("glyphon hello world")
            .build(&event_loop)
            .unwrap(),
    );
    let size = window.inner_size();
    let scale_factor = window.scale_factor();

    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::downlevel_defaults(),
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
        array_stride: mem::size_of::<Box>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &Box::buffer_desc(),
    };

    let resolution = Resolution { width: width as f32, height: height as f32 };
    let resolution_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Uniform Buffer"),
            contents: bytemuck::bytes_of(&resolution),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

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
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // Set up text renderer
    let mut font_system = FontSystem::new();
    let cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
    let text_renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

    let physical_width = (width as f64 * scale_factor) as f32;
    let physical_height = (height as f64 * scale_factor) as f32;

    buffer.set_size(&mut font_system, physical_width, physical_height);
    buffer.set_text(&mut font_system, "Hello world! 👋この動画の元になった作品ヽ༼ ຈل͜ຈ༽ ﾉヽ༼ ຈل͜ຈ༽ ﾉ\nヽ༼ ຈل͜ຈ༽\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system);

    let text_areas = vec![TextArea {
        buffer,
        left: 10.0,
        top: 10.0,
        scale: 1.0,
        bounds: TextBounds {
            left: 0,
            top: 0,
            right: 900,
            bottom: 660,
        },
        default_color: Color::rgb(255, 255, 255),
        depth: 0.0,
    }];

    let boxes = vec![
        Box { x0: -0.5, x1: 0.5, y0: -0.5, y1: 0.5 }
    ];

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
        boxes,
        gpu_vertex_buffer: vertex_buffer,
        resolution_buffer,
        bind_group,
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: Arc<Window>,
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    pub boxes: Vec<Box>,
    pub gpu_vertex_buffer: TypedGpuBuffer<Box>,
    pub render_pipeline: RenderPipeline,

    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_areas: Vec<TextArea>,

    pub resolution_buffer: wgpu::Buffer,
    pub bind_group: BindGroup,
}

impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => {
                    self.config.width = size.width;
                    self.config.height = size.height;
                    self.surface.configure(&self.device, &self.config);

                    let resolution = Resolution { width: size.width as f32, height: size.height as f32 };
                    self.queue.write_buffer(&self.resolution_buffer, 0, bytemuck::bytes_of(&resolution));

                    self.window.request_redraw();
                }
                WindowEvent::RedrawRequested => self.update_and_render(),
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }
    }

    pub fn update_and_render(&mut self) {
        self.gpu_vertex_buffer
            .queue_write(&self.boxes[..], &self.queue);

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
            )
            .unwrap();

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            const GREY: wgpu::Color = wgpu::Color { r: 0.027, g: 0.027, b: 0.027, a: 1.0 };
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
            let n = self.boxes.len() as u32;

            r_pass.set_pipeline(&self.render_pipeline);
            r_pass.set_bind_group(0, &self.bind_group, &[]);
            r_pass.set_vertex_buffer(0, self.gpu_vertex_buffer.slice(n));
            r_pass.draw(0..6, 0..n);
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
}

#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
#[repr(C)]
// Layout has to match the one in the shader.
pub struct Box {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
}
impl Box {
    pub fn buffer_desc() -> [VertexAttribute; 2] {
        return vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
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
