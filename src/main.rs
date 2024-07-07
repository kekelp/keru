pub mod helper;
pub mod ui;
pub mod canvas;

use canvas::{Canvas, EpicRotation};
use glam::dvec2;
use helper::*;
pub use ui::Id;
use ui::{Arrange, Axis::Y, Color, NodeParams, Ui, View};
use view_derive::derive_view;
use wgpu::TextureViewDescriptor;
use winit::{
    event::{Event, MouseButton}, event_loop::{EventLoop, EventLoopWindowTarget}, keyboard::KeyCode
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

fn init() -> (EventLoop<()>, State) {
    let (event_loop, window, instance, device, queue) =
        init_winit_and_wgpu(BASE_WIDTH, BASE_HEIGHT);
    let surface = instance.create_surface(window.clone()).unwrap();
    let config = configure_surface(&surface, &window, &device);

    let ui = Ui::new(&device, &config, &queue);
    let canvas = Canvas::new(BASE_WIDTH as usize, BASE_HEIGHT as usize, &device, &queue, &ui.uniform_buffer);
    let ctx = Context::new(window, surface, config, device, queue);

    let state = State {
        ctx,
        ui,
        counter_state: CounterState::new(),
        canvas,
    };

    return (event_loop, state);
}

pub struct State {
    pub ctx: Context,
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

impl State {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ctx.handle_events(event, target);
        let consumed = self.ui.handle_events(event, &self.ctx.queue);

        if ! consumed {
            self.canvas.handle_events(event, &self.ui.key_mods, &self.ctx.queue);
        }

        if is_redraw_requested(event) {
            self.update();
        }
    }

    pub fn update(&mut self) {
        let ui = &mut self.ui;

        ui.begin_tree();

        ui.update_gpu_time(&self.ctx.queue);

        // h_stack!(ui, CommandLineRow, {
        //     ui.add(CommandLine);
        // });

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

        self.update_canvas();
        
        self.render();

        // todo: why does this have to be here again?
        self.ui.part.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);

        let frame = self.ctx.surface.get_current_texture().unwrap();

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.ctx.device.create_command_encoder(&ENC_DESC);

        {
            let color_att = base_color_attachment(&view);
            let render_pass_desc = &base_render_pass_desc(&color_att);
            let mut render_pass = encoder.begin_render_pass(render_pass_desc);

            self.canvas.render(&mut render_pass, &mut self.ctx.queue);
            
            self.ui.render(&mut render_pass);

        }

        self.ctx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn update_canvas(&mut self) {
        self.canvas.draw_dots();

        self.zoom();
        self.rotate_and_pan();

        if self.canvas.end_stroke {
            self.canvas.mouse_dots.clear();
            self.canvas.end_stroke = false;
        }

        if self.canvas.need_backup {
            self.canvas.push_backup();
            self.canvas.need_backup = false;
        }

    }

    pub fn zoom(&mut self) {
        // todo, might be better to keep the last mouse pos *before the scrolling started*
        let mouse_before = self.canvas.screen_to_image(self.canvas.last_mouse_pos.x, self.canvas.last_mouse_pos.y);
        let mouse_before = dvec2(mouse_before.0, mouse_before.1);

        let (_x, y) = self.ctx.input.scroll_diff();

        let min_zoom = 0.01;
        let delta = y as f64 * 0.2;
        self.canvas.scale += delta ;
        if self.canvas.scale.y < min_zoom {
            self.canvas.scale.y = min_zoom;
        }
        if self.canvas.scale.x < min_zoom {
            self.canvas.scale.x = min_zoom;
        }

        let mouse_after = self.canvas.screen_to_image(self.canvas.last_mouse_pos.x, self.canvas.last_mouse_pos.y);
        let mouse_after = dvec2(mouse_after.0, mouse_after.1);

        let diff = mouse_after - mouse_before;
        
        // convert the mouse position diff (screen space) to image space.
        // --> only rotation and y invert
        let diff = dvec2(diff.x, -diff.y);
        let huh = self.canvas.rotation.inverse_vec();
        let diff = diff.rotate(huh);

        self.canvas.translation += diff;

        self.canvas.update_shader_transform(&self.ctx.queue);
    }

    pub fn rotate_and_pan(&mut self) {
        if self.ctx.input.mouse_held(MouseButton::Left) && self.ctx.input.key_held(KeyCode::Space) {

            let (x, y) = self.ctx.input.cursor_diff();
            if self.ctx.input.held_shift() {

                
                let new_angle = self.canvas.rotation.angle() + (y as f64 * 0.01);
                self.canvas.rotation = EpicRotation::new(new_angle);
            } else {

                let delta = dvec2(x as f64, y as f64);
                self.canvas.translation += delta / self.canvas.scale;

                self.canvas.update_shader_transform(&self.ctx.queue);
            }

            
        }
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
