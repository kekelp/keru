//! A single-line window/render loop, for experimentation and examples.
//! 
//! See the Counter example for a working demonstration,
//! 
//! The intended way to use Keru is with user-managed window loop and rendering. See the Painter example.
//! 
//! ### Example
//! 
//! ```rust
//! # use keru::*;
//! use keru::example_window_loop::*;
//! use keru::Ui;
//! 
//! #[derive(Default)]
//! pub struct State {
//!     // Custom program state
//! }
//! 
//! impl PureGuiLoop for State {
//!     fn declare_ui(&mut self, ui: &mut Ui) {
//!         // Custom GUI building logic, with access to your custom state (`self`) and the `Ui` object
//!     }
//! }
//! 
//! fn main() {
//!     let state = State::default();
//!     run_pure_gui_loop(state);
//! }
//! 
//! ```
//! 


use crate::*;
use crate::basic_window_loop::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

pub trait PureGuiLoop: Default {
    fn declare_ui(&mut self, ui: &mut Ui);
}

pub fn run_pure_gui_loop<S: PureGuiLoop>(state: S) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let ctx = Context::init();
    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);

    let mut full_state = State {
        user_state: state,
        ctx,
        ui,
    };

    let _ = event_loop.run_app(&mut full_state);
}

struct State<S> {
    user_state: S,
    ctx: Context,
    ui: Ui,
}

impl<S: PureGuiLoop> ApplicationHandler for State<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {  
        self.ctx.resume(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.handle_window_event(event_loop, _window_id, &event);
        self.ui.handle_events(&event);

        if let WindowEvent::RedrawRequested = &event {
            if self.ui.new_input() {
                println!("[{:?}] update", T0.elapsed());
                self.update();
            }

            if self.ui.needs_rerender() {
                println!("[{:?}] render", T0.elapsed());
                self.render();
            }

            // for some animations, we'll need to rerender several frames in a row without updating.
            if self.ui.needs_rerender() {
                self.ctx.window.request_redraw();
            }

        } else {            
            if self.ui.new_input() {
                self.ctx.window.request_redraw();
            }
        }
    }

    // fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
    //     // This will never get called if self.tick_mode is not TickAtScreenFps
    //     if let StartCause::ResumeTimeReached { .. } = cause {
    //         self.ctx.as_mut().unwrap().window.request_redraw();
    //     };
    // }
}

impl<S: PureGuiLoop> State<S> {
    pub fn update(&mut self) {
        self.ui.begin_tree();
        self.user_state.declare_ui(&mut self.ui);
        self.ui.finish_tree();
    }

    pub fn render(&mut self) {
        self.ui.prepare(&self.ctx.device, &self.ctx.queue);
        
        let mut frame = self.ctx.begin_frame();
        
        {
            let mut render_pass = frame.begin_render_pass(wgpu::Color::WHITE);
            self.ui.render(&mut render_pass);
        }
        
        frame.finish(&self.ctx);
    }
}
