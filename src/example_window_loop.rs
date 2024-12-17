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
//! impl ExampleLoop for State {
//!     fn declare_ui(&mut self, ui: &mut Ui) {
//!         // Custom GUI building logic, with access to your custom state (`self`) and the `Ui` object
//!     }
//! }
//! 
//! fn main() -> Result<(), WinitEventLoopError> {
//!     // One-line window + render loop
//!     run_with_example_loop::<State>()
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

    let mut full_state = State {
        user_state: state,
        ctx: None,
        ui: None,
    };

    let _ = event_loop.run_app(&mut full_state);
}

struct State<S> {
    user_state: S,
    ctx: Option<Context>,
    ui: Option<Ui>,
}

impl<S: PureGuiLoop> ApplicationHandler for State<S> {
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
        self.ctx.as_mut().unwrap().handle_window_event(event_loop, _window_id, &event);

        if let WindowEvent::RedrawRequested = &event {
            
            if self.ui.as_mut().unwrap().new_input() {
                println!("[{:?}] update", T0.elapsed());
                self.update();
            }

            if self.ui.as_mut().unwrap().needs_rerender() {
                println!("[{:?}] render", T0.elapsed());
                self.render();
            }

            // for some animations, we'll need to rerender several frames in a row without updating.
            if self.ui.as_mut().unwrap().needs_rerender() {
                self.ctx.as_mut().unwrap().window.request_redraw();
            }

        } else {
            
            let _consumed = self.ui.as_mut().unwrap().handle_events(&event);
            
            if self.ui.as_mut().unwrap().new_input() {
                self.ctx.as_mut().unwrap().window.request_redraw();
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
        let ui = self.ui.as_mut().unwrap();

        ui.begin_tree();
        self.user_state.declare_ui(ui);
        ui.finish_tree();
    }

    pub fn render(&mut self) {
        let ctx = self.ctx.as_mut().unwrap();
        let ui = self.ui.as_mut().unwrap();

        ui.prepare(&ctx.device, &ctx.queue);
        
        let mut frame = ctx.begin_frame();
        
        {
            let mut render_pass = frame.begin_render_pass(wgpu::Color::WHITE);
            ui.render(&mut render_pass);
        }
        
        frame.finish(&ctx);
    }
}
