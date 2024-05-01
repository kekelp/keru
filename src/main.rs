pub mod helper;
pub mod ui;
use helper::{
    base_color_attachment, base_render_pass_desc, base_surface_config, init_wgpu,
    init_winit_window, ENC_DESC,
};

pub use ui::Id;

use ui::{Color, NodeKey, NodeParams, Position, Size, Ui, Xy};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureViewDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

use std::{sync::Arc, time::Duration};

fn main() {
    let (event_loop, mut state) = init();

    event_loop
        .run(move |event, target| {
            state.handle_event(&event, target);
        })
        .unwrap();
}

pub const BASE_WIDTH: u32 = 1200;
pub const BASE_HEIGHT: u32 = 800;
pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

fn init() -> (EventLoop<()>, State<'static>) {
    let (event_loop, window) = init_winit_window(BASE_WIDTH as f64, BASE_HEIGHT as f64);
    let (instance, device, queue) = init_wgpu();

    let surface = instance.create_surface(window.clone()).unwrap();
    let size = window.inner_size();
    let config = base_surface_config(size.width, size.height, SWAPCHAIN_FORMAT);
    surface.configure(&device, &config);

    let ui = Ui::new(&device, &config, &queue);

    let state = State {
        window,
        surface,
        config,
        device,
        queue,
        ui,
        counter_state: CounterState::new(),
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: Arc<Window>,
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    pub ui: Ui,

    // app state
    pub counter_state: CounterState,
}

impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ui.handle_input_events(event);
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.resize(size),

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                self.update();
            }
            Event::AboutToWait => {
                self.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.counter_state.add(&mut self.ui);

        self.ui.finish_tree();
        self.ui.layout();

        self.counter_state.interact(&mut self.ui);

        self.ui.resolve_input();
        self.ui.build_buffers();

        self.render();

        self.ui.finish_frame();
    }

    pub fn render(&mut self) {
        if self.ui.needs_redraw() {
            self.ui.prepare(&self.device, &self.queue);

            let frame = self.surface.get_current_texture().unwrap();

            let view = frame.texture.create_view(&TextureViewDescriptor::default());
            let mut encoder = self.device.create_command_encoder(&ENC_DESC);

            {
                let color_att = base_color_attachment(&view);
                let render_pass_desc = &base_render_pass_desc(&color_att);
                let mut render_pass = encoder.begin_render_pass(render_pass_desc);

                self.ui.render(&mut render_pass);
            }

            self.queue.submit(Some(encoder.finish()));
            frame.present();
        } else {
            std::thread::sleep(Duration::from_millis(6));
        }
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.ui.resize(size, &self.queue);
        self.window.request_redraw();
    }

}

pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}

pub const CENTER_COLUMN: NodeKey = NodeKey::new(NodeParams::COLUMN, new_id!())
    .with_size_x(0.5)
    .with_position_x(Position::Center)
    .with_debug_name("Center column")
    .with_color(Color::BLUE);

pub const INCREASE_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Increase")
    .with_debug_name("Increase")
    .with_color(Color::BLUE);

pub const DECREASE_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Decrease")
    .with_debug_name("Decrease")
    .with_color(Color::BLUE);

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey::new(
    NodeParams {
        debug_name: "SHOW_COUNTER_BUTTON",
        static_text: Some("Show Counter"),
        clickable: false,
        color: Color::rgba(1.0, 0.3, 0.2, 0.6),
        size: Xy::new_symm(Size::PercentOfParent(0.2)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
    },
    new_id!(),
);

pub const COUNT_LABEL: NodeKey = NodeKey::new(NodeParams::LABEL, new_id!());

pub struct CounterState {
    pub count: i32,
    pub counter_mode: bool,
}
impl CounterState {
    pub fn new() -> Self {
        return CounterState {
            count: 0,
            counter_mode: true,
        };
    }

    pub fn add(&mut self, ui: &mut Ui) {
        floating_window!(ui, {
            add!(ui, CENTER_COLUMN, {
                if self.counter_mode {
                    add!(ui, INCREASE_BUTTON);
                    ui.update_color(INCREASE_BUTTON.id, count_color(self.count));

                    add!(ui, COUNT_LABEL);
                    ui.update_text(COUNT_LABEL.id, self.count);

                    add!(ui, DECREASE_BUTTON);
                }

                let text = match self.counter_mode {
                    true => "Hide counter",
                    false => "Show counter",
                };
                add!(ui, SHOW_COUNTER_BUTTON);
                ui.update_text(SHOW_COUNTER_BUTTON.id, text);
            });
        });
    }

    pub fn interact(&mut self, ui: &mut Ui) {
        if ui.is_clicked(INCREASE_BUTTON.id) {
            self.count += 1;
        }

        if ui.is_clicked(DECREASE_BUTTON.id) {
            self.count -= 1;
        }

        if ui.is_clicked(SHOW_COUNTER_BUTTON.id) {
            self.counter_mode = !self.counter_mode;
        }
    }
}
