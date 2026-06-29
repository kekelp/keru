#![windows_subsystem = "windows"]
mod canvas;
mod update_canvas;
mod paint_ui;
mod color_picker;
mod color_picker_render;
mod oklab;
mod window;

use canvas::Canvas;
use color_picker::ColorPicker;
use glam::dvec2;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;
use keru::*;
use window::*;
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

    let ctx = Context::new();

    let mut state = State::new(ctx);

    let _ = event_loop.run_app(&mut state);

    std::mem::forget(state);
}

pub struct State {
    pub ctx: Context,
    pub ui: Ui,
    pub color_picker: ColorPicker,
    pub canvas: Canvas,

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
        let consumed = self.ui.window_event(&event, &self.ctx.window);
        if !consumed {
            self.handle_canvas_event(&event);
        }

        if event == WindowEvent::RedrawRequested {
            self.update_and_render();
        }

        if event != WindowEvent::RedrawRequested || self.ui.should_rerender() {
            self.ctx.window.request_redraw();
        }
    }
}


impl State {
    fn new(ctx: Context) -> Self {
        let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);
        let canvas = Canvas::new(&ctx, &ctx.base_uniform_buffer);

        let color_picker = ColorPicker::new(&ctx);

        return State {
            ctx,
            ui,
            canvas,
            color_picker,
            show_ui: true,
            slider_value: 0.2,
        };
    }

    pub fn update_and_render(&mut self) {
        self.ui.begin_frame();
        if self.show_ui {    
            self.update_ui();
        }
        self.ui.finish_frame();

        self.update_canvas();

        let need_rerender = self.ui.should_rerender()
            || self.canvas.needs_rerender()
            || self.color_picker.need_rerender;

        if need_rerender {
            self.render();
        }
        self.color_picker.need_rerender = false;
    }

    pub fn render(&mut self) {
        // todo: if only the canvas needed rerender, we can skip ui.prepare(), and viceversa
        self.canvas.prepare(&self.ctx.queue);

        let mut frame = self.ctx.begin_frame();

        {
            let mut render_pass = frame.begin_render_pass(BACKGROUND_GREY);

            // Draw the canvas behind everything
            self.canvas.render(&mut render_pass);

            if self.show_ui {
                let window_size = [
                    self.ctx.surface_config.width as f32,
                    self.ctx.surface_config.height as f32,
                ];
                let commands = self.ui.render_commands().to_vec();
                self.ui.begin_custom_render();
                for command in commands {
                    match command {
                        RenderCommand::Keru(range) => {
                            self.ui.render_range(&mut render_pass, range);
                        }
                        RenderCommand::CustomRenderingArea { key, rect } => {
                            self.color_picker.render_custom(&mut render_pass, key, rect, window_size);
                        }
                    }
                }
                self.ui.finish_custom_render();
            }
        }

        self.ctx.window.pre_present_notify();
        self.ctx.finish_frame(frame);
    }



    pub fn handle_canvas_event(&mut self, event: &WindowEvent) {

        self.canvas.key_input.window_event(event);

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
                if *button == MouseButton::Middle {
                    self.canvas.middle_pressed = *state == ElementState::Pressed;
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let delta = dvec2(
                    position.x - self.canvas.last_mouse_pos.x,
                    position.y - self.canvas.last_mouse_pos.y,
                );
                self.canvas.last_mouse_pos = *position;

                if self.canvas.is_drawing && ! self.canvas.space {
                    self.canvas.mouse_dots.push(*position);
                }

                // Accumulate the pan/rotate drag delta for `rotate_and_pan`:
                // left button while Space is held, middle button otherwise.
                let panning = if self.canvas.space {
                    self.canvas.is_drawing
                } else {
                    self.canvas.middle_pressed
                };
                if panning {
                    self.canvas.pan_drag_delta += delta;
                }
            },
            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                if ! is_synthetic && event.state.is_pressed() {
                    if let Key::Character(new_char) = &event.logical_key {
                    match new_char.as_str() {
                        "z" => {
                            if self.ui.key_input().key_mods().control_key() {
                                self.canvas.undo();
                            }
                        },
                        "Z" => {
                            if self.ui.key_input().key_mods().control_key() {
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
