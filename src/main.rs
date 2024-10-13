// #![windows_subsystem = "windows"]

pub mod pixels_on_screen;
pub mod ui_node_params;
pub mod canvas;
pub mod main_canvas;
pub mod main_ui;
pub mod ui_texture_atlas;
pub mod ui_math;
pub mod ui;
pub mod ui_render;
pub mod ui_layout;
pub mod ui_interact;
pub mod ui_text;

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
    pub count_state: CounterState,
    pub canvas: Canvas,

    pub info_visible: bool,
    pub slider_value: f32,
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
        
        self.render();
    }

    pub fn render(&mut self) {
        if ! self.ui.need_rerender() {
            return;
        }

        println!("Render");
            
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
            count_state: CounterState::default(),
            canvas,
            info_visible: true,
            slider_value: 0.4,
        };
    }
}
