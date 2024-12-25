#![windows_subsystem = "windows"]
mod canvas;
mod update_canvas;
mod paint_ui;
mod color_picker;
mod color_picker_render;
mod oklab;

use canvas::*;
use color_picker::ColorPicker;
use glam::dvec2;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;
use keru::*;
use keru::basic_window_loop::*;
use winit::event::*;
use winit::keyboard::*;

pub const WINDOW_NAME: &str = "Keru Paint Example";

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::", log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let ctx = Context::init();

    let mut state = State::new(ctx);

    let _ = event_loop.run_app(&mut state);
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

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.ctx.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.window_event(event_loop, _window_id, &event);
        let consumed = self.ui.handle_event(&event);
        if !consumed {
            self.handle_canvas_event(&event);
        }

        if event == WindowEvent::RedrawRequested {
            self.update_and_render();
        }

        // In the paint program, input events almost always cause an update/rerender (update hovered pixel info, ...)
        // Instead of bothering to track if the canvas absorbs the event, we do this unconditionally.
        // Should still track it, tbh.
        if event != WindowEvent::RedrawRequested || self.ui.needs_rerender() {
            self.ctx.window.request_redraw();
        }
    }
}


impl State {
    fn new(ctx: Context) -> Self {
        let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
        let canvas = Canvas::new(&ctx, ui.base_uniform_buffer());

        #[node_key] pub const COLOR_PICKER_1: NodeKey;
        let color_picker = ColorPicker::new(COLOR_PICKER_1, &ctx, ui.base_uniform_buffer());

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

    pub fn update_and_render(&mut self) {
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
        }
    }

    pub fn render(&mut self) {
        // todo: if only the canvas needed rerender, we can skip ui.prepare(), and viceversa
        self.canvas.prepare(&self.ctx.queue);
        
        self.color_picker.prepare(&mut self.ui, &self.ctx.queue);

        let mut frame = self.ctx.begin_frame();

        {
            let mut render_pass = frame.begin_render_pass(BACKGROUND_GREY);

            self.canvas.render(&mut render_pass);
            self.ui.render(&mut render_pass, &self.ctx.device, &self.ctx.queue);
            
            if ! self.ui.debug_mode() && self.show_ui {
                self.color_picker.render(&mut render_pass);
            }
        }

        self.ctx.window.pre_present_notify();
        frame.finish(&self.ctx);
    }



    pub fn handle_canvas_event(&mut self, event: &WindowEvent) {

        self.canvas.mouse_input.handle_event(event);
        self.canvas.key_input.handle_event(event);
        
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
