use keru::*;
use wgpu::*;
use std::sync::Arc;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};

struct Application {
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    _queue: Queue,
    config: SurfaceConfiguration,
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

        Self { window, surface, device, _queue: queue, config, ui, count: 0 }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
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
    // Right now we assume that for desktop applications there's only one Resume event at the beginning, so it's okay to create the whole state here.
    // If this wasn't the case, we'd probably need to recreate just the winit window and the graphics context, and keep the rest.
    // If the Ui ends up holding a Weak<Window> or similar, that would need to be updated too. 
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
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
                    state.ui.autorender(&state.surface, wgpu::Color::BLACK);
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