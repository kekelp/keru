pub mod helper;
pub mod ui;
use helper::{
    base_color_attachment, base_render_pass_desc, configure_surface, init_winit_and_wgpu,
    WgpuWindow, ENC_DESC,
};

use rustc_hash::FxHasher;
pub use ui::Id;

use ui::{Arrange, Axis::{X, Y}, Color, NodeKey, NodeParams, Position, Ui, UiDefaults, UiDefaultsParam, UiId, UiIdParam};
use wgpu::TextureViewDescriptor;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
};

use std::{any::TypeId, hash::{Hash, Hasher}, time::Duration};

use crate::ui::{TreeTraceEntry, NODE_ROOT_ID};

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

        ui.content_changed = true;

        ui.update_time();
        ui.update_gpu_time(&self.window.queue);

        ui.tree_trace.clear();
        ui.tree_trace_defaults.clear();

        h_stack!(ui, &COMMAND_LINE_ROW, {
            ui.add2(&COMMAND_LINE);
        });

        frame!(ui, {
            h_stack!(ui, &CENTER_ROW, {
                v_stack!(ui, {
                    if self.counter_state.counter_mode {
                        let new_color = count_color(self.counter_state.count);
                        ui.add2(&INCREASE_BUTTON).set_color(new_color);

                        ui.add2(&COUNT_LABEL).set_text(self.counter_state.count);

                        ui.add2(&DECREASE_BUTTON);
                    }
                });

                v_stack!(ui, {
                    let text = match self.counter_state.counter_mode {
                        true => "Hide counter",
                        false => "Show counter",
                    };
                    ui.add2(&SHOW_COUNTER_BUTTON).set_text(text);
                });
            });
        });

        // todo dont clone
        let mut current_parent_id = NODE_ROOT_ID;
        for (i, e) in ui.tree_trace.clone().iter().enumerate() {
            match e {
                TreeTraceEntry::Node(id) => {
                    let defaults = ui.tree_trace_defaults[i].unwrap();
                    let key = NodeKey {
                        id: *id,
                        defaults,
                    };
                    ui.update_hashmap(&key, Some(current_parent_id));
                },
                TreeTraceEntry::SetParent(id) => {
                    current_parent_id = *id;
                },
            }
        }

        // println!("{:?}", ui.tree_trace);
        // println!("");

        ui.finish_tree();

        ui.layout();
        ui.resolve_mouse_input();

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
        
        self.ui.finish_frame();
    }

    pub fn render(&mut self) {
        if self.ui.needs_redraw() {
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
        } else {
            std::thread::sleep(Duration::from_millis(6));
        }
    }
}

pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}

pub const CENTER_ROW: NodeKey = NodeKey::new(NodeParams::H_STACK, new_id!())
    .with_debug_name("Center column")
    // .with_stack(X, Arrange::End)
    .with_color(Color::BLUE);

pub const INCREASE_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Increase")
    .with_debug_name("Increase")
    .with_color(Color::BLUE);

pub const DECREASE_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Decrease")
    .with_debug_name("Decrease")
    .with_color(Color::BLUE);

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey::new(NodeParams::BUTTON, new_id!())
    .with_static_text("Show Counter")
    .with_debug_name("SHOW_COUNTER_BUTTON")
    .with_color(Color::BLUE);

pub const COUNT_LABEL: NodeKey = NodeKey::new(NodeParams::LABEL, new_id!());

pub const COMMAND_LINE: NodeKey = NodeKey::new(NodeParams::TEXT_INPUT, new_id!())
    .with_debug_name("Command line")
    .with_size_y(0.1)
    .with_static_text("高38道ょつ準傷に債健の🤦🏼‍♂️🚵🏻‍♀️");

pub const COMMAND_LINE_ROW: NodeKey = NodeKey::new(NodeParams::H_STACK, new_id!())
    .with_debug_name("Center column")
    .with_size_y(0.95)
    .with_size_x(0.8)
    .with_stack(Y, Arrange::Start)
    .with_color(Color::BLUE);

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
