pub mod helper;
pub mod ui;
use helper::{
    base_color_attachment, base_render_pass_desc, base_surface_config, init_wgpu,
    init_winit_window, ENC_DESC,
};
pub use ui::Id;

use ui::{Color, LayoutMode, NodeKey, Ui};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureViewDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::Window,
};

use std::sync::Arc;

#[rustfmt::skip]
fn main() {
    let (event_loop, mut state) = init();

    event_loop.run(
        move |event, target| {
            state.handle_event(&event, target);
        }
    ).unwrap();
}

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 800;
pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

fn init() -> (EventLoop<()>, State<'static>) {
    let (event_loop, window) = init_winit_window(WIDTH as f64, HEIGHT as f64);
    let (instance, device, queue) = init_wgpu();

    let surface = instance.create_surface(window.clone()).unwrap();
    let config = base_surface_config(WIDTH, HEIGHT);
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
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::RedrawRequested => {
                    self.update();
                    self.window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        let ui = &mut self.ui;

        floating_window!((ui) {

            div!(ui, COUNT_LABEL.with_text(self.count));

            column!((ui) {

                let text = match self.counter_mode {
                    true => &"Hide counter",
                    false => &"Show counter",
                };
                div!(ui, SHOW_COUNTER_BUTTON.with_text(text));

                if self.counter_mode {
                    let color = Color { r: 0.1 * (self.count as f32), g: 0.0, b: 0.0, a: 1.0 };
                    div!(ui, INCREASE_BUTTON.with_color(color));
                }

            });

        });

        self.ui.layout();
        // self.resolve_input();

        if self.ui.is_clicked(INCREASE_BUTTON) {
            self.count += 1;
        }

        if self.ui.is_clicked(SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
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




pub const INCREASE_BUTTON: NodeKey = NodeKey::button()
    .with_static_text("Increase")
    .with_color(Color {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    })
    .with_layout_x(LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    })
    .with_layout_y(LayoutMode::Fixed {
        start: 400,
        len: 100,
    })
    .with_id(id!());

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey::button()
    .with_static_text("Increase")
    .with_color(Color {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    })
    .with_layout_x(LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    })
    .with_layout_y(LayoutMode::Fixed {
        start: 100,
        len: 100,
    })
    .with_id(id!());

pub const COUNT_LABEL: NodeKey = NodeKey::button()
    .with_color(Color {
        r: 0.1,
        g: 0.3,
        b: 0.9,
        a: 0.6,
    })
    .with_id(id!());