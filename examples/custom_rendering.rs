//! Custom Rendering Example with Custom Shaders
//!
//! This example demonstrates the custom rendering system in Keru with actual custom WGPU rendering.
//! It shows how to integrate your own render pipeline with custom shaders between Keru's UI elements.
//!
//! Key features demonstrated:
//! - Custom vertex and fragment shaders
//! - Custom render pipeline setup
//! - Interleaving custom shader rendering between UI text labels
//! - Proper Z-ordering where custom content appears between UI elements

use keru::*;
use wgpu::*;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};

struct Application {
    state: Option<State>,
}

// To do any kind of wgpu rendering by hand, we first have to go through all of wgpu's boilerplate. 
struct CustomRenderer {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    time: f32,
}

impl CustomRenderer {
    fn new(device: &Device, surface_format: TextureFormat) -> Self {
        // Custom shader that draws an animated gradient quad
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Custom Shader"),
            source: ShaderSource::Wgsl(include_str!("custom_rendering_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Custom Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX_FRAGMENT,
                range: 0..16, // 4 floats: x, y, width, height
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Custom Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x2,
                    }],
                }],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Quad vertices (unit square)
        let vertices: &[f32] = &[
            0.0, 0.0,
            1.0, 0.0,
            0.0, 1.0,
            1.0, 1.0,
        ];

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Custom Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            time: 0.0,
        }
    }

    fn render(&self, render_pass: &mut RenderPass, rect: &XyRect, screen_size: (f32, f32)) {
        // Convert normalized coordinates to screen space
        let x = rect.x[0] * screen_size.0;
        let y = rect.y[0] * screen_size.1;
        let width = (rect.x[1] - rect.x[0]) * screen_size.0;
        let height = (rect.y[1] - rect.y[0]) * screen_size.1;

        // Push constants: position and size in pixels
        let push_constants: [f32; 4] = [x, y, width, height];

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&push_constants),
        );
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
    }
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    _queue: Queue,
    config: SurfaceConfiguration,
    ui: Ui,
    count: i32,
    custom_renderer: CustomRenderer,
    last_frame_time: std::time::Instant,
}

impl State {
    fn new(window: Arc<Window>, instance: Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::PUSH_CONSTANTS,
            required_limits: Limits { max_push_constant_size: 16, ..Default::default() },
            ..Default::default()
        })).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

        // When possible, using a linear color format for the surface results in better color blending.
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| ! f.is_srgb())
            .copied().unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut ui = Ui::new(&device, &queue, &config);
        ui.enable_auto_wakeup(window.clone());

        let custom_renderer = CustomRenderer::new(&device, surface_format);

        Self {
            window,
            surface,
            device,
            _queue: queue,
            config,
            ui,
            count: 0,
            custom_renderer,
            last_frame_time: std::time::Instant::now(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    #[node_key] const CUSTOM_RECT: NodeKey;
    #[node_key] const LABEL_TOP: NodeKey;
    #[node_key] const LABEL_BOTTOM: NodeKey;
    #[node_key] const BUTTON: NodeKey;

    fn update_ui(&mut self) {

        // Text label positioned at top
        let count_text = format!("Text BELOW custom shader (Count: {})", self.count);
        let label_top = LABEL
            .color(keru::Color::rgba(255, 255, 255, 230))
            .padding(20)
            .position(Position::Static(Len::Pixels(50)), Position::Static(Len::Pixels(100)))
            .text(&count_text)
            .key(Self::LABEL_TOP);

        // Custom rendered rectangle - this will be drawn with our custom shader
        // It's invisible in Keru's rendering
        let custom_rect = DEFAULT
            .invisible()
            .size(Size::Pixels(400), Size::Pixels(200))
            .position(Position::Static(Len::Pixels(100)), Position::Static(Len::Pixels(150)))
            .custom_render(true)
            .key(Self::CUSTOM_RECT);

        // Text label positioned at bottom - will render ABOVE the custom shader
        let label_bottom = LABEL
            .color(keru::Color::rgba(255, 255, 100, 230))
            .padding(20)
            .position(Position::Static(Len::Pixels(150)), Position::Static(Len::Pixels(300)))
            .text("Text ABOVE custom shader")
            .key(Self::LABEL_BOTTOM);

        // Button to increment counter
        let button = keru::BUTTON
            .position(Position::Static(Len::Pixels(200)), Position::Static(Len::Pixels(400)))
            .text("Click to increment")
            .key(Self::BUTTON);

        self.ui.add(label_top);
        self.ui.add(custom_rect);
        self.ui.add(label_bottom);
        self.ui.add(button);

        if self.ui.is_clicked(Self::BUTTON) {
            self.count += 1;
        }
    }

    fn custom_render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // Update animation
        let now = std::time::Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.custom_renderer.update(dt);

        self.ui.begin_custom_render(render_pass);

        // Get the render plan
        let render_plan = self.ui.render_plan().to_vec();
        let screen_size = (self.config.width as f32, self.config.height as f32);

        for command in render_plan {
            match command {
                RenderCommand::Keru(range) => {
                    // Render the regular UI elements for this range.
                    self.ui.render_range(render_pass, range);
                }
                RenderCommand::CustomRenderingArea { key, rect } => {
                    // Do our custom rendering. If there were multiple custom rendered rects, we could tell them apart by key.
                    if key == Self::CUSTOM_RECT {
                        self.custom_renderer.render(render_pass, &rect, screen_size);
                    }
                }
            }
        }

        self.ui.finish_custom_render();
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        window.set_ime_allowed(true);
        let instance = Instance::new(&InstanceDescriptor::default());
        let state = State::new(window, instance);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.ui.window_event(&event, &state.window);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if state.ui.should_update() {
                    state.ui.begin_frame();
                    state.update_ui();
                    state.ui.finish_frame();
                }
                if state.ui.should_rerender() {
                    // Instead of using autorender, we do custom rendering
                    let output = state.surface.get_current_texture().unwrap();
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("custom render encoder"),
                    });

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("custom render pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                        state.custom_render(&mut render_pass);
                    }

                    state.ui.submit_commands(encoder.finish());
                    output.present();
                }
            }
            _ => {}
        }

        if state.ui.should_request_redraw() {
            state.window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application { state: None };
    let _ = event_loop.run_app(&mut app);
}
