pub mod helper;
pub mod ui;
pub mod canvas;

use canvas::Canvas;
use helper::*;
pub use ui::Id;
use ui::{Arrange, Axis::Y, Color, NodeParams, Ui, View};
use view_derive::derive_view;
use wgpu::TextureViewDescriptor;
use winit::{
    event::Event,
    event_loop::{EventLoop, EventLoopWindowTarget},
};

fn main() {
    let (event_loop, mut state) = init();

    event_loop
        .run(move |event, target| {
            state.handle_event(&event, target);
        })
        .unwrap();
}

pub const BASE_WIDTH: f64 = 1200.0;
pub const BASE_HEIGHT: f64 = 800.0;

fn init() -> (EventLoop<()>, State<'static>) {
    let (event_loop, window, instance, device, queue) =
        init_winit_and_wgpu(BASE_WIDTH, BASE_HEIGHT);
    let surface = instance.create_surface(window.clone()).unwrap();
    let config = configure_surface(&surface, &window, &device);

    let ui = Ui::new(&device, &config, &queue);
    let canvas = Canvas::new(BASE_WIDTH as usize, BASE_HEIGHT as usize, &device);
    let window = WgpuWindow::new(window, surface, config, device, queue);

    let state = State {
        window,
        ui,
        counter_state: CounterState::new(),
        canvas,
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: WgpuWindow<'window>,
    pub ui: Ui,
    // app state
    pub counter_state: CounterState,
    pub canvas: Canvas,
}

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
}
pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}

impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.window.handle_events(event, target);
        self.ui.handle_events(event, &self.window.queue);

        self.canvas.handle_events(event);

        if is_redraw_requested(event) {
            self.update();
        }
    }

    pub fn update(&mut self) {
        let ui = &mut self.ui;

        ui.begin_tree();

        ui.update_gpu_time(&self.window.queue);

        h_stack!(ui, CommandLineRow, {
            ui.add(CommandLine);
        });

        ui.finish_tree();

        if ui.is_clicked(IncreaseButton) {
            self.counter_state.count += 1;
        }

        if ui.is_clicked(DecreaseButton) {
            self.counter_state.count -= 1;
        }

        if ui.is_clicked(ShowCounterButton) {
            self.counter_state.counter_mode = !self.counter_state.counter_mode;
        }

        self.ui.build_buffers();

        self.canvas.update();
        
        self.render();

        // todo: why does this have to be here again?
        self.ui.part.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.ui.prepare(&self.window.device, &self.window.queue);

        let frame = self.window.surface.get_current_texture().unwrap();

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.window.device.create_command_encoder(&ENC_DESC);

        {
            let color_att = base_color_attachment(&view);
            let render_pass_desc = &base_render_pass_desc(&color_att);
            let mut render_pass = encoder.begin_render_pass(render_pass_desc);

            self.canvas.render(&mut render_pass, &mut self.window.queue);
            
            self.ui.render(&mut render_pass);

        }

        self.window.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

#[derive_view(NodeParams::H_STACK.color(Color::BLUE))]
pub struct CenterRow;

#[derive_view(NodeParams::BUTTON.text("Increase").color(Color::GREEN))]
pub struct IncreaseButton;

#[derive_view(NodeParams::BUTTON.text("Decrease").color(Color::RED))]
pub struct DecreaseButton;

#[derive_view(NodeParams::BUTTON.text("Show Counter").color(Color::rgba(0.5, 0.1, 0.7, 0.7)))]
pub struct ShowCounterButton;

#[derive_view(NodeParams::LABEL)]
pub struct CountLabel;

#[derive_view(
    NodeParams::H_STACK
    .size_y(0.95)
    .size_x(0.8)
    .stack(Y, Arrange::End)
    .color(Color::BLUE)
)]
pub struct CommandLineRow;

#[derive_view(NodeParams::TEXT_INPUT.text("高38道ょつヽ༼ຈل͜ຈ༽ﾉ準傷に債健の🤦🏼‍♂️🚵🏻‍♀️").size_y(0.1))]
pub struct CommandLine;

#[allow(dead_code)]
pub fn useless_counter(ui: &mut Ui, counter_state: &mut CounterState) {
    margin!(ui, {
        h_stack!(ui, CenterRow, {
            v_stack!(ui, {
                if counter_state.counter_mode {
                    let new_color = count_color(counter_state.count);
                    ui.add(IncreaseButton).set_color(new_color);

                    let count = &counter_state.count.to_string();
                    ui.add(CountLabel).set_text(count);

                    ui.add(DecreaseButton);
                }
            });

            v_stack!(ui, {
                let text = match counter_state.counter_mode {
                    true => "Hide counter",
                    false => "Show counter",
                };
                ui.add(ShowCounterButton).set_text(text);
            });
        });
    });
}
