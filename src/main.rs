pub mod helper;
pub mod ui;
use helper::{
    base_color_attachment, base_render_pass_desc, base_surface_config, init_wgpu,
    init_winit_window, ENC_DESC,
};
pub use ui::Id;

use ui::{Color, NodeKey, NodeParams, Ui, Xy, Size, Position};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureViewDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

use std::sync::Arc;

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
    pub ui: Ui,

    // app state
    pub count: i32,
    pub counter_mode: bool,
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
        let ui = &mut self.ui;

        floating_window!(ui, {
            // ui.add(COUNT_LABEL.with_text(self.count));

            div!(ui, CENTER_COLUMN, {

                let text = match self.counter_mode {
                    true => "Hide counter",
                    false => "Show counter",
                };
                // ui.add(SHOW_COUNTER_BUTTON.with_text(text));
                div!(ui, SHOW_COUNTER_BUTTON.with_static_text(text));

                if self.counter_mode {
                    let red = 0.1 * (self.count as f32);
                    let color = Color::rgba(red, 0.1, 0.2, 0.8);
                    ui.add(INCREASE_BUTTON.with_color(color));

                    // column!(ui, {
                    //     ui.add(INCREASE_BUTTON.sibling(2).with_color(color));
                    // });
                }
            });

            // column!(ui, {
            //     for l in ["X", "Y", "Z", "W", "Y"] {
            //         ui.add(LETTER_BUTTON.sibling(l).with_text(l));
            //     }
            // });
        });

        self.ui.layout();
        // self.resolve_input();

        if self.ui.is_clicked(INCREASE_BUTTON) || self.ui.is_clicked(INCREASE_BUTTON.sibling(2)) {
            self.count += 1;
        }

        if self.ui.is_clicked(SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
        }

        for l in ["X", "Y", "Z", "W", "Y"] {
            if self.ui.is_clicked(LETTER_BUTTON.sibling(l)) {
                println!(" {:?}", l);
            }
        }

        self.ui.build_buffers();

        self.render();

        self.ui.current_frame += 1;
        self.ui.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
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
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.ui.resize(size, &self.queue);
        self.window.request_redraw();
    }
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

pub const LETTER_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Letter")
    .with_debug_name("Letter")
    .with_color(Color::LIGHT_BLUE);

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey::new(
    NodeParams {
        debug_name: "SHOW_COUNTER_BUTTON",
        static_text: Some("Show Counter"),
        dyn_text: None,
        clickable: false,
        color: Color::rgba(1.0, 0.3, 0.2, 0.6),
        size: Xy::new_symm(Size::PercentOfParent(0.2)),
        position: Xy::new_symm(Position::Start { padding: 5 }),
        container_mode: None,
    },
    new_id!(),
);

pub const COUNT_LABEL: NodeKey = NodeKey::new(NodeParams::LABEL, new_id!());
