#![windows_subsystem = "windows"]
mod canvas;
mod main_canvas;
mod paint_ui;
mod color_picker;
mod color_picker_render;
mod oklab;

use blue::T0;
use canvas::*;
use color_picker::ColorPicker;
use glam::dvec2;
use winit::{error::EventLoopError, event::Event, event_loop::EventLoopWindowTarget};
use blue::Ui;
use blue::basic_window_loop::*;
use winit::event::*;
use winit::keyboard::*;

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

    pub format_scratch: String,

    pub show_ui: bool,
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
            show_ui: true,
            slider_value: 0.2,
            format_scratch: String::with_capacity(100),
        };
    }

    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ctx.handle_events(event, target);
        let consumed = self.ui.handle_events(event, &self.ctx.queue);
        if !consumed {
            self.handle_events(event);
        }

        if event.is_redraw_requested() {
            self.update();
        }
    }

    pub fn update(&mut self) {
        self.ui.begin_tree();
        if self.show_ui {    
            self.update_ui();
        }
        self.ui.finish_tree();

        self.update_canvas();

        let need_rerender = self.ui.needs_rerender()
            || self.canvas.needs_rerender()
            || self.color_picker.needs_rerender();

        if need_rerender {
            self.render();
        } else {
            self.ctx.sleep_until_next_frame();
        }
    }

    pub fn render(&mut self) {
        // todo: if only the canvas needed rerender, we can skip ui.prepare(), and viceversa
        self.canvas.prepare(&self.ctx.queue);
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);
        
        self.color_picker.prepare(&mut self.ui, &self.ctx.queue);

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



    pub fn handle_events(&mut self, full_event: &winit::event::Event<()>) {

        self.canvas.input.update(full_event);
        
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Left {

                        self.canvas.is_drawing = *state == ElementState::Pressed;
                        if ! self.canvas.space {
                            match state {
                                ElementState::Pressed => {
                                    self.canvas.mouse_dots.push(self.canvas.last_mouse_pos);
                                },
                                ElementState::Released => {
                                    // do the backup on release so that it doesn't get in the way computationally speaking
                                    self.canvas.end_stroke = true;
                                    self.canvas.need_backup = true;
                                },
                            }
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    self.canvas.last_mouse_pos = *position;

                    if self.canvas.is_drawing && ! self.canvas.space {
                        self.canvas.mouse_dots.push(*position);
                    }
                },
                WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                    // println!("  {:?}", event );
                    if ! is_synthetic && event.state.is_pressed() {
                        if let Key::Character(new_char) = &event.logical_key {
                        match new_char.as_str() {
                            "z" => {
                                if self.ui.key_mods().control_key() {
                                    self.canvas.undo();
                                }
                            },
                            "Z" => {
                                if self.ui.key_mods().control_key() {
                                    self.canvas.redo();
                                }
                            },
                                _ => {},
                            }
                        }
                    }

                    if ! is_synthetic {
                        if let Key::Named(NamedKey::Space) = &event.logical_key {
                            self.canvas.space = event.state.is_pressed();
                        }
                        if let Key::Named(NamedKey::Tab) = &event.logical_key {
                            if event.state.is_pressed() {
                                self.show_ui = ! self.show_ui;
                            }
                        }
                    }
                },
                // todo, this sucks actually.
                WindowEvent::Resized(size) => {
                    self.canvas.width = size.width as usize;
                    self.canvas.height = size.height as usize;
                    self.canvas.update_shader_transform(&mut self.ctx.queue);
                },

                WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                            self.canvas.scroll.y += *y as f64;
                        },
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            self.canvas.scroll += dvec2(pos.x, pos.y);
                        },
                    }
                }

                _ => {}
            }
        }
    }

}
