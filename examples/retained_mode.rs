/// A counter example that uses retained mode.
///
/// The initial GUI tree is declared just once at startup.
/// On subsequent frames, we don't redeclare anything:
/// we just run the minimal imperative code for changing state and modifying the retained GUI tree.
/// Is this really a good idea? Who knows!
///
/// This example is a bit longer because it can't use the run_example_loop helper,
/// so it includes all the winit and wgpu boilerplate.

use keru::*;
use keru::node_library::*;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[node_key] const SHOW: NodeKey;
#[node_key] const INCREASE: NodeKey;
#[node_key] const DECREASE: NodeKey;
#[node_key] const COUNT_LABEL: NodeKey;
#[node_key] const COUNTER_AREA: NodeKey;

struct Application {
    state: Option<State>,
}

struct State {
    ui: Ui,
    count: i32,
    show: bool,
    // winit/wgpu state
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
}

impl State {
    fn build_initial_tree(&mut self) {
        self.ui.begin_frame();

        let v_stack = V_STACK.animate_position(true);
        let show_button = BUTTON.color(Color::RED).text("Hide Counter").key(SHOW);
        let counter_area = V_STACK.key(COUNTER_AREA);

        self.ui.add(v_stack).nest(|| {
            self.ui.add(show_button);
            self.ui.add(counter_area).nest(|| {
                self.add_counter_children();
            });
        });

        self.ui.finish_frame();
    }

    fn add_counter_children(&mut self) {
        let count_str = self.count.to_string();
        let increase_button = BUTTON.color(Color::RED).text("Increase").key(INCREASE);
        let count_label = LABEL.text(&count_str).key(COUNT_LABEL);
        let decrease_button = BUTTON.text("Decrease").key(DECREASE);

        self.ui.add(increase_button);
        self.ui.add(count_label);
        self.ui.add(decrease_button);
    }

    fn run_frame(&mut self) {
        self.ui.begin_retained_mode_frame();

        if self.ui.is_clicked(SHOW) {
            self.show = !self.show;
            if self.show {
                // If you don't like unwrap() and similar things, 
                // it's probably better to stick to declarative mode!
                // Every mutation on the tree involves a leap of faith, as you have to assume 
                // what the state of the tree is at that point.
                // This is the fundamental downside of retained mode:
                // with every imperative mutation, the state of the tree gets farther and farther 
                // from anything that you can see written in the code explicitly.
                // To understand what's going on, you have to keep track of all the mutations so far,
                // and play them back inside your head.
                // But maybe for a fairly static GUI it's fine.
                self.ui.jump_to_node(COUNTER_AREA).unwrap().nest(|| {
                    self.add_counter_children();
                });

                self.ui.get_node_mut(SHOW).unwrap().set_text("Hide Counter");
            } else {
                self.ui.get_node_mut(COUNTER_AREA).unwrap().clear_all_children();
                self.ui.get_node_mut(SHOW).unwrap().set_text("Show Counter");
            }
        }

        if self.show {
            if self.ui.is_clicked(INCREASE) {
                self.count += 1;
            }
            if self.ui.is_clicked(DECREASE) {
                self.count -= 1;
            }
            self.ui.get_node_mut(COUNT_LABEL).unwrap().set_text(&self.count.to_string());
        }
        
        self.ui.finish_retained_mode_frame();
    }
}

// winit/wgpu boilerplate...
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let state = State::new(event_loop, window, instance);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.ui.window_event(&event, &state.window);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if state.ui.current_frame() == 0 {
                    state.build_initial_tree();
                } else if state.ui.should_update() {
                    state.run_frame();
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
// more boilerplate...
impl State {
    fn new(event_loop: &ActiveEventLoop, window: Arc<Window>, instance: wgpu::Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| !f.is_srgb())
            .copied().unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut ui = Ui::new(&device, &queue, &config);
        ui.register_window(event_loop, window.clone());

        Self { window, surface, device, config, ui, count: 0, show: true }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application { state: None };
    let _ = event_loop.run_app(&mut app);
}
