use keru::*;
use std::sync::Arc;
use wgpu::*;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window};

struct State {
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    instance: Instance,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    depth_texture: Texture,
    ui: Ui,
    count: i32,
}

impl State {
    fn new() -> Self {
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::PUSH_CONSTANTS,
            required_limits: Limits { max_push_constant_size: 8, ..Default::default() },
            ..Default::default()
        }, None)).unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 1920,
            height: 1080,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture = device.create_texture(&TextureDescriptor {
            size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth"),
            view_formats: &[],
        });

        let ui = Ui::new(&device, &queue, &config);

        Self { window: None, surface: None, instance, device, queue, config, depth_texture, ui, count: 0 }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.config);
        }
        self.depth_texture = self.device.create_texture(&TextureDescriptor {
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth"),
            view_formats: &[],
        });
    }

    fn update_ui(&mut self) {
        #[node_key] const INCREASE: NodeKey;
        
        let button = BUTTON
            .color(keru::Color::RED)
            .text("Increase")
            .key(INCREASE);

        self.ui.v_stack().nest(|| {
            self.ui.add(button);
            self.ui.label(&self.count.to_string());
        });

        if self.ui.is_clicked(INCREASE) {
            self.count += 1;
        }
    }

    fn render(&mut self) {
        let surface_texture = self.surface.as_ref().unwrap().get_current_texture().unwrap();
        let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());
        let depth_view = self.depth_texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations { load: LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }), store: StoreOp::Store },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(Operations { load: LoadOp::Clear(1.0), store: StoreOp::Store }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.ui.render(&mut render_pass, &self.device, &self.queue);
        }
        
        self.queue.submit([encoder.finish()]);
        surface_texture.present();
    }
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let surface = self.instance.create_surface(window.clone()).unwrap();
        
        self.window = Some(window);
        self.surface = Some(surface);
        
        let size = self.window.as_ref().unwrap().inner_size();
        self.resize(size.width, size.height);
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            self.ui.window_event(&event, window);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if self.ui.needs_update() {
                    self.ui.begin_frame();
                    self.update_ui();
                    self.ui.finish_frame();
                }
                if self.ui.needs_rerender() {
                    self.render();
                }
            }
            _ => {}
        }
        
        if self.ui.event_loop_needs_to_wake() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut state = State::new();
    let _ = event_loop.run_app(&mut state);
}