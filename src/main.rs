// #![windows_subsystem = "windows"]

pub mod pixels_on_screen;
pub mod ui;
pub mod node_params;
pub mod canvas;
pub mod main_canvas;
pub mod main_ui;
pub mod texture_atlas;

use glam::dvec2;
use pixels_on_screen::*;
use canvas::*;
use ui::*;
use main_ui::CounterState;

pub const BASE_WIDTH: u32 = 1350;
pub const BASE_HEIGHT: u32 = 850;
pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.014,
    g: 0.014 + 0.002,
    b: 0.014,
    a: 1.0,
};
pub const WINDOW_NAME: &str = "BLUE";

fn main() -> Result<(), EventLoopError> {
    let (ctx, event_loop) = Context::init(BASE_WIDTH, BASE_HEIGHT, WINDOW_NAME);

    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
    let canvas = Canvas::new(&ctx, &ui.base_uniform_buffer);
    
    let mut state = State {
        ctx,
        ui,
        counter_state: CounterState::new(),
        canvas,

        info_visible: true,
        slider_value: 500.0,
    };

    event_loop.run(move |event, target| {
        state.handle_event(&event, target);
    })?;

    Ok(())
}

pub struct State {
    pub ctx: Context,
    pub ui: Ui,
    pub counter_state: CounterState,
    pub canvas: Canvas,

    pub info_visible: bool,
    pub slider_value: f32,
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
        self.update_ui();
        self.update_canvas();
        
        self.canvas.scroll = dvec2(0.0, 0.0);
        self.render();
    }

    pub fn render(&mut self) {
        self.canvas.prepare(&self.ctx.queue);
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);

        let mut frame = self.ctx.begin_frame();

        {
            let mut render_pass = frame.begin_render_pass(BACKGROUND_COLOR);
            self.canvas.render(&mut render_pass);
            self.ui.render(&mut render_pass);
        }
        
        frame.finish(&self.ctx.queue);
    }
}
