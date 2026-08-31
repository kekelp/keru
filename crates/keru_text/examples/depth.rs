use keru_text::*;
use std::sync::Arc;
use wgpu::*;
use wgpu::util::DeviceExt;
use winit::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application { state: None })
        .unwrap();
}

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,

    text: Text,

    triangle_pipeline: RenderPipeline,
    triangle_vertex_buffer: Buffer,
    
    depth_texture: Texture,
    depth_view: TextureView,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TriangleVertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor::default())).unwrap();
        
        let surface = instance.create_surface(window.clone()).expect("Create surface");
        let surface_config = surface.get_default_config(&adapter, physical_size.width, physical_size.height).unwrap();
        surface.configure(&device, &surface_config);

        // Create depth texture
        let depth_format = TextureFormat::Depth32Float;
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width: physical_size.width,
                height: physical_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: depth_format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        let triangle_depth_stencil_state = DepthStencilState {
            format: depth_format,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::Less),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        let text_depth_stencil_state = DepthStencilState {
            format: depth_format,
            depth_write_enabled: Some(false),
            depth_compare: Some(CompareFunction::Less),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        let text_depth = 0.5;
        let mut text = Text::new_with_params(&device, &queue, surface_config.format, Some(text_depth_stencil_state), TextRendererParams::default());
        let _text_handle = text.add_text_box(
            "Text rendering supports basic depth testing, but this isn't enough to draw multiple semitransparent objects both behind and in front of text. The third triangle is rendered in a separate draw call.    Text rendering supports basic depth testing, but this isn't enough to draw multiple semitransparent objects both behind and in front of text. The third triangle is rendered in a separate draw call.    Text rendering supports basic depth testing, but this isn't enough to draw multiple semitransparent objects both behind and in front of text. The third triangle is rendered in a separate draw call.    Text rendering supports basic depth testing, but this isn't enough to draw multiple semitransparent objects both behind and in front of text. The third triangle is rendered in a separate draw call.    ",
            Some((50.0, 50.0)),
            (700.0, 300.0),
            text_depth
        );

        let triangle_shader_source = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 1.0);
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

        // Create triangle pipeline
        let triangle_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: ShaderSource::Wgsl(triangle_shader_source.into()),
        });

        let triangle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: None,
            vertex: VertexState {
                module: &triangle_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(VertexBufferLayout {
                    array_stride: std::mem::size_of::<TriangleVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x4],
                })],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &triangle_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(triangle_depth_stencil_state),
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let triangle_vertices = [
            // behind text (opaque red)
            TriangleVertex { position: [-0.7, 0.9, 0.8], color: [1.0, 0.0, 0.0, 1.0] },
            TriangleVertex { position: [-0.3, 0.9, 0.8], color: [1.0, 0.0, 0.0, 1.0] },
            TriangleVertex { position: [-0.5, 0.5, 0.8], color: [1.0, 0.0, 0.0, 1.0] },
            
            // in front of text (opaque green)
            TriangleVertex { position: [0.3, 0.9, 0.2], color: [0.0, 1.0, 0.0, 1.0] },
            TriangleVertex { position: [0.7, 0.9, 0.2], color: [0.0, 1.0, 0.0, 1.0] },
            TriangleVertex { position: [0.5, 0.5, 0.2], color: [0.0, 1.0, 0.0, 1.0] },

            // Semitransparent blue
            TriangleVertex { position: [-0.2,  0.0, 0.9], color: [0.1, 0.1, 1.0, 0.7] },
            TriangleVertex { position: [ 0.2,  0.0, 0.9], color: [0.1, 0.1, 1.0, 0.7] },
            TriangleVertex { position: [ 0.0, -0.4, 0.9], color: [0.1, 0.1, 1.0, 0.7] },
        ];

        let triangle_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("triangle vertex buffer"),
            contents: bytemuck::cast_slice(&triangle_vertices),
            usage: BufferUsages::VERTEX,
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            text,
            triangle_pipeline,
            triangle_vertex_buffer,
            depth_texture,
            depth_view,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            // Recreate depth texture with new size
            self.depth_texture = self.device.create_texture(&TextureDescriptor {
                label: Some("depth texture"),
                size: Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_view = self.depth_texture.create_view(&TextureViewDescriptor::default());
        }
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            other => panic!("Failed to get current surface texture: {other:?}"),
        };
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        // Prepare text rendering
        self.text.prepare_all();

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.triangle_pipeline);
            render_pass.set_vertex_buffer(0, self.triangle_vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);

            self.text.render(&mut render_pass);

            render_pass.set_pipeline(&self.triangle_pipeline);
            render_pass.set_vertex_buffer(0, self.triangle_vertex_buffer.slice(..));
            render_pass.draw(6..9, 0..1);

        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
    }

    fn window_event(&mut self, event: WindowEvent) {
        self.text.handle_event(&event, &self.window);

        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
}

struct Application {
    state: Option<State>,
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .with_resizable(false);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window.set_ime_allowed(true);

        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            event => {
                if let Some(state) = &mut self.state {
                    state.window_event(event);
                    state.window.request_redraw();
                }
            }
        }
    }
}