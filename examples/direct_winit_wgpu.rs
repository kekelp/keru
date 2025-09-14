use keru::*;
use std::sync::Arc;
use wgpu::*;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window};

struct Application {
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    depth_texture: Texture,
    ui: Ui,
    count: i32,
}

impl State {
    fn new(window: Arc<Window>, instance: Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            // todo: remove
            required_features: Features::PUSH_CONSTANTS,
            required_limits: Limits { max_push_constant_size: 8, ..Default::default() },
            ..Default::default()
        }, None)).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // The depth texture shouldn't be needed anymore in future versions of Keru, which will use a more advanced renderer.
        let depth_texture = device.create_texture(&TextureDescriptor {
            size: Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: Some("depth"),
            view_formats: &[],
        });

        let ui = Ui::new(&device, &queue, &config);

        Self { window, surface, device, queue, config, depth_texture, ui, count: 0 }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
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
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let instance = Instance::new(InstanceDescriptor::default());
        let state = State::new(window, instance);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        
        state.ui.window_event(&event, &state.window);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if state.ui.needs_update() {
                    state.ui.begin_frame();
                    state.update_ui();
                    state.ui.finish_frame();
                }
                if state.ui.needs_rerender() {
                    state.ui.create_render_pass_and_render(
                        &state.surface,
                        &state.depth_texture,
                        &state.device,
                        &state.queue,
                    );
                }
            }
            _ => {}
        }
        
        if state.ui.event_loop_needs_to_wake() {
            state.window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application { state: None };
    let _ = event_loop.run_app(&mut app);
}