#![windows_subsystem = "windows"]
mod canvas;
mod main_canvas;
mod main_ui;
mod color_picker;
mod color_picker_render;

use canvas::*;
use color_picker::ColorPicker;
use color_picker_render::*;
use winit::{error::EventLoopError, event::Event, event_loop::EventLoopWindowTarget};
use blue::Ui;
use blue::basic_window_loop::*;

pub const WINDOW_NAME: &str = "BLUE";

fn main() -> Result<(), EventLoopError> {
    let (ctx, event_loop) = Context::init(1350, 850, "BLUE");

    let mut state = State::new(ctx);

    event_loop.run(move |event, target| {
        state.handle_event(&event, target);
    })?;

    Ok(())
}

pub struct State {
    pub ctx: Context,
    pub ui: Ui,
    pub color_picker: ColorPicker,
    pub canvas: Canvas,

    pub info_visible: bool,
    pub slider_value: f32,
}

impl State {
    fn new(ctx: Context) -> Self {
        let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
        let canvas = Canvas::new(&ctx, ui.base_uniform_buffer());
        let color_picker = ColorPicker::new(&ctx, ui.base_uniform_buffer());

        return State {
            ctx,
            ui,
            canvas,
            color_picker,
            info_visible: true,
            slider_value: 0.4,
        };
    }

    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ctx.handle_events(event, target);
        let consumed = self.ui.handle_events(event, &self.ctx.queue);
        if !consumed {
            self.canvas.handle_events(event, self.ui.key_mods(), &self.ctx.queue);
        }

        if event.is_redraw_requested() {
            self.update();
        }
    }

    pub fn update(&mut self) {
        self.declare_ui();
        self.update_canvas();

        if self.ui.needs_rerender() || self.canvas.needs_rerender() {
            self.render();
        } else {
            self.ctx.sleep_until_next_frame();
        }
    }

    pub fn render(&mut self) {
        // todo: if only the canvas needed rerender, we can skip ui.prepare(), and viceversa
        self.canvas.prepare(&self.ctx.queue);
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);
        
        self.color_picker.coords = [self.ui.get_node(ColorPicker::HUE_WHEEL).unwrap().get_inner_rect()];        
        self.color_picker.update_coordinates(&self.ctx.queue);

        let mut frame = self.ctx.begin_frame();

        {
            let mut render_pass = frame.begin_render_pass(BACKGROUND_GREY);

            self.canvas.render(&mut render_pass);
            self.ui.render(&mut render_pass);
            
            if ! self.ui.debug_mode() {
                self.color_picker.render(&mut render_pass);
            }
            

        }

        self.ctx.window.pre_present_notify();
        frame.finish(&self.ctx.queue);
    }
}
