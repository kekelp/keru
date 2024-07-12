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
use winit::{event::Event, event_loop::EventLoopWindowTarget};


fn main() -> Result<(), EventLoopError> {
    let (event_loop, mut state) = init();

    event_loop.run(move |event, target| {
        state.handle_event(&event, target);
    })?;

    Ok(())
}

pub const BASE_WIDTH: f64 = 1350.0;
pub const BASE_HEIGHT: f64 = 850.0;

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
    pub counter_state: CounterState,
    pub canvas: Canvas,
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
            let mut render_pass = frame.begin_render_pass();
            self.canvas.render(&mut render_pass);
            self.ui.render(&mut render_pass);
        }
        
        frame.finish(&self.ctx.queue);
    }
}
