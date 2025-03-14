//! A very simple way to start a `winit`/`wgpu` window loop and to draw a Keru GUI inside it.
//! 
//! See the [`run_example_loop`] function for an example.

use crate::*;
use crate::basic_window_loop::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

pub use basic_window_loop::basic_env_logger_init;

pub trait ExampleLoop: Default {
    fn update_ui(&mut self, ui: &mut Ui);
}

struct FullState<S> {
    user_state: S,
    ctx: Context,
    ui: Ui,
}

/// A single-line window/render loop, for experimentation and examples.
/// 
/// See the Counter example for a working example,
/// 
/// The intended way to use Keru is with a user-managed window and rendering loop.
/// 
/// ### Example
/// 
/// ```no_run
/// # use keru::*;
/// use keru::example_window_loop::*;
/// use keru::Ui;
/// 
/// #[derive(Default)]
/// pub struct State {
///     // Custom program state
/// }
/// 
/// impl ExampleLoop for State {
///     fn update_ui(&mut self, ui: &mut Ui) {
///         // Custom GUI building logic, with access to your custom state (`self`) and the `Ui` object
///     }
/// }
/// 
/// fn main() {
///     let state = State::default();
///     run_example_loop(state);
/// }
/// ```
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
        self.ctx.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.window_event(event_loop, _window_id, &event);
        self.ui.window_event(&event);

        if event == WindowEvent::RedrawRequested {
            if self.ui.needs_update() {
                self.ui.begin_frame();
                self.user_state.update_ui(&mut self.ui);
                self.ui.finish_frame();
            }

            if self.ui.needs_rerender() {
                self.ctx.render_ui(&mut self.ui);
            }
        }
                
        if self.ui.event_loop_needs_to_wake() {
            self.ctx.window.request_redraw();
        }
    }
}
