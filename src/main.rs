pub mod helper;
pub mod ui;
use helper::{
    base_color_attachment, base_render_pass_desc, configure_surface, init_winit_and_wgpu,
    WgpuWindow, ENC_DESC,
};

pub use ui::Id;

use ui::{Arrange, Axis::Y, Color, NodeKey, NodeParams, Ui};
use wgpu::TextureViewDescriptor;
use winit::{
    event::{Event, WindowEvent},
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

    let state = State {
        window: WgpuWindow::new(window, surface, config, device, queue),
        ui,
        counter_state: CounterState::new(),
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: WgpuWindow<'window>,
    pub ui: Ui,
    // app state
    pub counter_state: CounterState,
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


impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.window.handle_events(event, target);
        self.ui.handle_events(event, &self.window.queue);

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::RedrawRequested = event {
                self.update();
            }
        }
    }

    pub fn update(&mut self) {
        let ui = &mut self.ui;

        ui.begin_tree();
        
        ui.update_gpu_time(&self.window.queue);

        h_stack!(ui, &COMMAND_LINE_ROW, {
            ui.add(&COMMAND_LINE);
        });

        margin!(ui, {
            h_stack!(ui, &CENTER_ROW, {
                v_stack!(ui, {
                    if self.counter_state.counter_mode {
                        let new_color = count_color(self.counter_state.count);
                        ui.add(&INCREASE_BUTTON).set_color(new_color);

                        ui.add(&COUNT_LABEL).set_text(&self.counter_state.count.to_string());

                        ui.add(&DECREASE_BUTTON);
                    }
                });

                v_stack!(ui, {
                    let text = match self.counter_state.counter_mode {
                        true => "Hide counter",
                        false => "Show counter",
                    };
                    ui.add(&SHOW_COUNTER_BUTTON).set_text(text);
                });
            });
        });

        ui.finish_tree();


        if ui.is_clicked(INCREASE_BUTTON.id) {
            self.counter_state.count += 1;
        }
        
        if ui.is_clicked(DECREASE_BUTTON.id) {
            self.counter_state.count -= 1;
        }
        
        if ui.is_clicked(SHOW_COUNTER_BUTTON.id) {
            self.counter_state.counter_mode = !self.counter_state.counter_mode;
        }
        
        self.ui.build_buffers();

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

            self.ui.render(&mut render_pass);
        }

        self.window.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}

pub const CENTER_ROW: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::H_STACK)
    .with_color(Color::BLUE);

pub const INCREASE_BUTTON: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::BUTTON)
    .with_static_text("Increase")
    .with_color(Color::BLUE);

pub const DECREASE_BUTTON: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::BUTTON)
    .with_static_text("Decrease")
    .with_color(Color::BLUE);

pub const SHOW_COUNTER_BUTTON: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::BUTTON)
    .with_static_text("Show Counter")
    .with_color(Color::BLUE);

pub const COUNT_LABEL: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::LABEL);

pub const COMMAND_LINE: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::TEXT_INPUT)
    .with_size_y(0.1)
    .with_static_text("高38道ょつ準傷に債健の🤦🏼‍♂️🚵🏻‍♀️");

pub const COMMAND_LINE_ROW: NodeKey = unique_node_key!()
    .with_defaults(NodeParams::H_STACK)
    .with_size_y(0.95)
    .with_size_x(0.8)
    .with_stack(Y, Arrange::End)
    .with_color(Color::BLUE);
