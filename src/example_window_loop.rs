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
//!     fn update_ui(&mut self, ui: &mut Ui) {
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
use crate::*;
use crate::basic_window_loop::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

pub trait ExampleLoop: Default {
    fn update_ui(&mut self, ui: &mut Ui);
}

struct FullState<S> {
    user_state: S,
    ctx: Context,
    ui: Ui,
}

pub fn run_example_loop<S: ExampleLoop>(state: S) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let ctx = Context::init();
    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);

    let mut full_state = FullState {
        user_state: state,
        ctx,
        ui,
    };

    let _ = event_loop.run_app(&mut full_state);
}

impl<S: ExampleLoop> ApplicationHandler for FullState<S> {
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
                self.ui.begin_tree();
                self.user_state.update_ui(&mut self.ui);
                self.ui.finish_tree();
            }

            if self.ui.needs_rerender() {
                println!("[{:?}] render", T0.elapsed());
                self.ctx.render_ui(&mut self.ui);
            }

            // If there is an animation playing, ui.needs_rerender() will return true even if we just rendered.
            // In that case, call request_redraw and go for another iteration of the loop.
            // This works right only thanks to render_ui() waiting for vsync. 
            // Doing a similar thing for multiple update()s in a row without rendering wouldn't be as simple, and would need to use winit's ControlFlow::WaitUntil. 
            // (Is it even correct in the rendering-only case? It probably goes to the next iteration, and immediately starts sleeping waiting for vsync. That's "real sleep" in which, if new input events arrive, the loop won't be able to wake up and handle it, right? That's significantly dumber than WaitUntil.)
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
