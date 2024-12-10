use crate::*;
use crate::basic_window_loop::*;

pub trait ExampleLoop: Default {
    fn declare_ui(&mut self, ui: &mut Ui);
}

pub fn run_with_example_loop<S: ExampleLoop>() -> Result<(), EventLoopError> {
    let (ctx, event_loop) = Context::init(1350, 850, "Keru Example");
    
    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
    
    let mut state = State {
        user_state: S::default(),
        ctx,
        ui,
    };

    event_loop.run(move |event, target| {
        state.handle_event(&event, target);
    })?;

    Ok(())
}

struct State<S> {
    user_state: S,
    ctx: Context,
    ui: Ui,
}

impl<S: ExampleLoop> State<S> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ctx.handle_events(event, target);
        let _consumed = self.ui.handle_events(event, &self.ctx.queue);

        if event.is_redraw_requested() {
            self.update();
        }
    }

    pub fn update(&mut self) {
        self.ui.begin_tree();
        self.user_state.declare_ui(&mut self.ui);
        self.ui.finish_tree();
        
        if self.ui.needs_rerender() {
            self.render();
        } else {
            self.ctx.sleep_until_next_frame();
        }
    }

    pub fn render(&mut self) {           
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);
        
        let mut frame = self.ctx.begin_frame();
        
        {
            let mut render_pass = frame.begin_render_pass(wgpu::Color::WHITE);
            self.ui.render(&mut render_pass);
        }
        
        frame.finish(&self.ctx.queue);
    }
}
