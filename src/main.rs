pub mod helper;
pub mod ui;
pub mod canvas;
pub mod main_canvas;
pub mod main_ui;

use canvas::*;
use helper::*;
use main_ui::CounterState;
pub use ui::Id;
use ui::Ui;
use winit::{event::Event, event_loop::{self, EventLoopWindowTarget}};


fn main() -> Result<(), EventLoopError> {
    let (ctx, event_loop) = Context::new2(BASE_WIDTH, BASE_HEIGHT);

    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
    let canvas = Canvas::new(&ctx, &ui.base_uniform_buffer);
    
    let mut state = State {
        ctx,
        ui,
        counter_state: CounterState::new(),
        canvas,

        info_visible: true,
    };

    event_loop.run(move |event, target| {
        state.handle_event(&event, target);
    })?;

    Ok(())
}

pub const BASE_WIDTH: f64 = 1350.0;
pub const BASE_HEIGHT: f64 = 850.0;
pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.014,
    g: 0.014 + 0.002,
    b: 0.014,
    a: 1.0,
};

pub struct State {
    pub ctx: Context,
    pub ui: Ui,
    pub counter_state: CounterState,
    pub canvas: Canvas,

    pub info_visible: bool,
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
        
        self.render();

        // todo: why does this have to be here again?
        self.ui.part.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.canvas.prepare(&mut self.ctx.queue);
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
