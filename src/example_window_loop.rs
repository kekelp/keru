//! A very simple way to start a `winit`/`wgpu` window loop and to draw a Keru GUI inside it.
//!
//! See the [`run_example_loop`] function for an example.

use crate::basic_window_loop::*;
use crate::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

pub use basic_window_loop::basic_env_logger_init;

/// A single-line window/render loop, for experimentation and examples.
///
/// This function is only meant for examples and quick experimentation. The intended way to use Keru is with a user-managed window and rendering loop. See the `window_loop.rs` in the repository for a concise example of that. 
/// 
/// `state` is the program's state, and can be any type.
/// `update_fn` is a function that reads the current `state`, updates a Keru [`Ui`], and can optionally modify the `state`. 
/// ### Example
///
/// ```no_run
/// use keru::example_window_loop::*;
/// use keru::*;
/// 
/// #[derive(Default)]
/// pub struct State {
///     pub count: i32,
/// }
/// 
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     #[node_key] const INCREASE: NodeKey;
/// 
///     let increase_button = BUTTON
///         .color(Color::RED)
///         .text("Increase")
///         .key(INCREASE);
/// 
///     ui.v_stack().nest(|| {
///         ui.add(increase_button);
///         ui.label(&state.count.to_string());
///     });
/// 
///     if ui.is_clicked(INCREASE) {
///         state.count += 1;
///     }
/// }
/// 
/// fn main() {
///     let state = State::default();
///     run_example_loop(state, update_ui);
/// }
/// ```
/// 
/// `update_fn` can also be a method on the state `T`.
/// 
pub fn run_example_loop<T>(state: T, update_fn: fn(&mut T, &mut Ui)) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let ctx = Context::new();
    let ui = Ui::new(&ctx.device, &ctx.queue, &ctx.surface_config);

    let mut app = AppWrapper {
        ctx,
        ui,
        state,
        update_fn,
    };

    let _ = event_loop.run_app(&mut app);
}

struct AppWrapper<T> {
    state: T,
    update_fn: fn(&mut T, &mut Ui),
    ctx: Context,
    ui: Ui,
}

impl<T> ApplicationHandler for AppWrapper<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.ctx.resumed(event_loop);
        self.ui.enable_cursor_blink_auto_wakeup(self.ctx.window.clone());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.ctx.window_event(event_loop, _window_id, &event);
        self.ui.window_event(&event, &self.ctx.window);

        if event == WindowEvent::RedrawRequested {
            if self.ui.should_update() {
                self.ui.begin_frame();
                (self.update_fn)(&mut self.state, &mut self.ui);
                self.ui.finish_frame();
            }

            if self.ui.should_rerender() {
                self.ctx.render_ui(&mut self.ui);
            }
        }

        if self.ui.should_request_redraw() {
            self.ctx.window.request_redraw();
        }
    }
}
