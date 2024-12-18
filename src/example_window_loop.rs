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

use std::sync::Arc;

use crate::*;
use crate::basic_window_loop::*;
use winit::application::ApplicationHandler;
pub use winit::error::EventLoopError as WinitEventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

pub trait PureGuiLoop: Default {
    fn declare_ui(&mut self, ui: &mut Ui);
}

pub fn run_pure_gui_loop<S: PureGuiLoop>(state: S) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut full_state = WinitFriendlyState {
        user_state: state,
        ctx: None,
        ui: None,
    };

    let _ = event_loop.run_app(&mut full_state);
}

struct WinitFriendlyState<S> {
    user_state: S,
    ctx: Option<Context>,
    ui: Option<Ui>,
}

struct State<'a, S> {
    user_state: &'a mut S,
    ctx: &'a mut Context,
    ui: &'a mut Ui,
}

impl<S> WinitFriendlyState<S> {
    fn unwrap<'a>(&'a mut self) -> State<'a, S> {
        return State {
            user_state: &mut self.user_state,
            ctx: self.ctx.as_mut().unwrap(),
            ui: self.ui.as_mut().unwrap(),
        }
    }
} 

impl<S: PureGuiLoop> ApplicationHandler for WinitFriendlyState<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        
        let ctx = Context::init(1350, 860, window);
        let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);

        self.ctx = Some(ctx);
        self.ui = Some(ui);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut _self = self.unwrap();
        _self.ctx.handle_window_event(event_loop, _window_id, &event);

        if let WindowEvent::RedrawRequested = &event {
            
            if _self.ui.new_input() {
                println!("[{:?}] update", T0.elapsed());
                _self.update();
            }

            if _self.ui.needs_rerender() {
                println!("[{:?}] render", T0.elapsed());
                _self.render();
            }

            // for some animations, we'll need to rerender several frames in a row without updating.
            if _self.ui.needs_rerender() {
                _self.ctx.window.request_redraw();
            }

        } else {
            
            let _consumed = _self.ui.handle_events(&event);
            
            if _self.ui.new_input() {
                _self.ctx.window.request_redraw();
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

impl<'a, S: PureGuiLoop> State<'a, S> {
    pub fn update(&mut self) {
        self.ui.begin_tree();
        self.user_state.declare_ui(self.ui);
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
