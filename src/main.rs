// #![windows_subsystem = "windows"]

pub mod pixels_on_screen;
pub mod ui;
pub mod node_params;
pub mod canvas;
pub mod main_canvas;
pub mod main_ui;
pub mod texture_atlas;
pub mod add_parent_manual;
pub mod add_parent_closure;
pub mod math;
pub mod render;

use glam::dvec2;
use pixels_on_screen::*;
use canvas::*;
use ui::*;
use main_ui::CounterState;

pub const BASE_WIDTH: u32 = 1350;
pub const BASE_HEIGHT: u32 = 850;
pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.037,
    g: 0.037 + 0.002,
    b: 0.037,
    a: 1.0,
};
pub const WINDOW_NAME: &str = "BLUE";

fn main() -> Result<(), EventLoopError> {
    let (ctx, event_loop) = Context::init(BASE_WIDTH, BASE_HEIGHT, WINDOW_NAME);
    
    let mut state = State::new(ctx);

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
    pub slider_value2: f32,
    pub slider_value3: f32,
    pub slider_value4: f32,
    pub slider_value5: f32,
}


impl State {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {        
        self.ctx.handle_events(event, target);
        
        let consumed = self.ui.handle_events(event, &self.ctx.queue);

        if ! consumed {
            self.canvas.handle_events(event, &self.ui.sys.key_mods, &self.ctx.queue);
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

    fn new(ctx: Context) -> Self {
        let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
        let canvas = Canvas::new(&ctx, &ui.sys.base_uniform_buffer);
        
        return State {
            ctx,
            ui,
            counter_state: CounterState::new(),
            canvas,
    
            info_visible: true,
            slider_value: 500.0,
            slider_value2: 450.0,
            slider_value3: 450.0,
            slider_value4: 450.0,
            slider_value5: 450.0,
        };
    }
}
