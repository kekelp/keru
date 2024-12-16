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
use std::time::{Duration, Instant};

use crate::*;
use crate::basic_window_loop::*;
use winit::application::ApplicationHandler;
pub use winit::error::EventLoopError as WinitEventLoopError;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

pub trait ExampleLoop: Default {
    fn declare_ui(&mut self, ui: &mut Ui);
}

pub fn run_with_example_loop<S: ExampleLoop>(state: S) {
    let event_loop = EventLoop::new().unwrap();
        
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

impl<S: ExampleLoop> ApplicationHandler for State<S> {
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
        let _consumed = self.ui.as_mut().unwrap().handle_events(&event);

        self.ctx.as_mut().unwrap().handle_window_event(event_loop, _window_id, &event);

        if let WindowEvent::RedrawRequested = &event {
            self.tick(&event_loop);
        }

        // if self.ui.as_mut().unwrap().needs_rerender() {
        //     println!("  {:?}", event);
        //     self.ctx.as_mut().unwrap().window.request_redraw();
        // }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.ctx.as_mut().unwrap().window.request_redraw();
        };
    }
}

impl<S: ExampleLoop> State<S> {
    pub fn tick(&mut self, event_loop: &ActiveEventLoop) {
        let ui = self.ui.as_mut().unwrap();

        
        println!("[{:?}] update", T0.elapsed());
        
        // if self.ui.needs_update() {
            ui.begin_tree();
            self.user_state.declare_ui(ui);
            ui.finish_tree();
        // }

        
        if ui.needs_rerender() {
            println!("[{:?}] render", T0.elapsed());
            self.render();
            event_loop.set_control_flow(ControlFlow::Poll);
            self.ctx.as_mut().unwrap().window.request_redraw();
        }
        else {
            let refresh_rate = self.ctx.as_mut().unwrap().window.current_monitor().unwrap().video_modes().next().unwrap().refresh_rate_millihertz();        
            let frame_time_micros = (1_000_000_000 / refresh_rate) as u64;
            let sleep_time = Duration::from_micros(frame_time_micros);
            let wake_time = Instant::now() + sleep_time;
            event_loop.set_control_flow(ControlFlow::WaitUntil(wake_time));
        }

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
