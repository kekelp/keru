//! A very simple way to start a `winit`/`wgpu` window loop and to draw a Keru GUI inside it.
//!
//! See the [`run_example_loop`] function for an example.

use crate::*;
use std::sync::Arc;
use std::time::Instant;
use wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Initialize env_logger with default settings for examples.
pub fn basic_env_logger_init() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::", log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();
}

/// A single-line window/render loop, for experimentation and examples.
///
/// This function is only meant for examples and quick experimentation. The intended way to use Keru is with a user-managed window and rendering loop. See the `window_loop.rs` example in the repository for a concise example of that.
///
/// `state` is the program's state, and can be any type.
/// `update_fn` is a function that reads the current `state`, updates a Keru [`Ui`], and can optionally modify the `state`.
/// ### Example
///
/// ```no_run
/// use keru::example_window_loop::*;
/// use keru::*;
///
/// #[derive(Default)]
/// pub struct State {
///     pub count: i32,
/// }
///
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     #[node_key] const INCREASE: NodeKey;
///
///     let increase_button = BUTTON
///         .color(Color::RED)
///         .text("Increase")
///         .key(INCREASE);
///
///     ui.v_stack().nest(|| {
///         ui.add(increase_button);
///         ui.label(&state.count.to_string());
///     });
///
///     if ui.is_clicked(INCREASE) {
///         state.count += 1;
///     }
/// }
///
/// fn main() {
///     let state = State::default();
///     run_example_loop(state, update_ui);
/// }
/// ```
///
/// `update_fn` can also be a method on the state `T`.
///
pub fn run_example_loop<T>(user_state: T, update_fn: fn(&mut T, &mut Ui)) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = Application {
        state: None,
        user_state,
        update_fn,
    };

    let _ = event_loop.run_app(&mut app);
}

struct Application<T> {
    state: Option<State>,
    user_state: T,
    update_fn: fn(&mut T, &mut Ui),
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    _queue: Queue,
    config: SurfaceConfiguration,
    ui: Ui,
}

impl State {
    fn new(window: Arc<Window>, instance: Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
            required_limits: Limits { max_push_constant_size: 8, ..Default::default() },
            memory_hints: MemoryHints::MemoryUsage,
            ..Default::default()
        })).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

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

        let ui = Ui::new(&device, &queue, &config);

        Self { window, surface, device, _queue: queue, config, ui }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

impl<T> ApplicationHandler for Application<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let instance = Instance::new(&InstanceDescriptor::default());
        let mut state = State::new(window, instance);
        state.ui.register_window(state.window.clone());
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.ui.window_event(&event, &state.window);

        // match event {
        //     WindowEvent::CloseRequested => event_loop.exit(),
        //     WindowEvent::Resized(size) => state.resize(size.width, size.height),
        //     WindowEvent::RedrawRequested => {
        //         let frame_start = Instant::now();

        //         if state.ui.should_update() {
        //             state.ui.begin_frame();
        //             (self.update_fn)(&mut self.user_state, &mut state.ui);
        //             state.ui.finish_frame();
        //         }
        //         if state.ui.should_rerender() {
        //             state.ui.autorender(&state.surface, wgpu::Color::BLACK);
        //         }

        //         let frame_time = frame_start.elapsed();
        //         log::info!("Time since last frame: {:?}", frame_time);
        //     }
        //     _ => {}
        // }

        // if state.ui.should_request_redraw() {
        //     state.window.request_redraw();
        // }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let frame_start = Instant::now();

                state.ui.begin_frame();
                (self.update_fn)(&mut self.user_state, &mut state.ui);
                state.ui.finish_frame();
            
                state.ui.autorender(&state.surface, wgpu::Color::BLACK);

                let frame_time = frame_start.elapsed();
                log::info!("Time since last frame: {:?}", frame_time);
            }
            _ => {}
        }

        state.window.request_redraw();

    }
}
