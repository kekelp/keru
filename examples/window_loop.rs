// This example shows how to run Keru with a user-managed Winit/Wgpu loop.
// It shows how you can run a Keru ui without giving up control of your main loo... well, without giving up control of your Winit loop.
// It still uses the helper Context struct defined in keru::basic_window_loop. If you need more control or just want to look inside, you should copy the struct and its methods and customize them.

use keru::*;
use keru::basic_window_loop::Context;

pub struct State {
    pub winit_wgpu_ctx: Context,
    pub ui: keru::Ui,

    pub count: i32,
}

fn main() {
    // Create the winit event loop
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    // Create the Winit window and the Wgpu instance, device, queue, etc.
    let winit_wgpu_context = Context::new();

    // create the Keru Ui.
    let ui = keru::Ui::new(&winit_wgpu_context.device, &winit_wgpu_context.queue, &winit_wgpu_context.surface_config);

    let mut state = State {
        winit_wgpu_ctx: winit_wgpu_context,
        ui,
        count: 0,
    };

    let _ = event_loop.run_app(&mut state);
}

impl State {
    fn update_ui(&mut self) {
        #[node_key] const INCREASE: NodeKey;
        
        let increase_button = BUTTON
            .color(Color::RED)
            .text("Increase")
            .key(INCREASE);
    
        self.ui.v_stack().nest(|| {
            self.ui.add(increase_button);
            self.ui.label(&self.count.to_string());
        });

        if self.ui.is_clicked(INCREASE) {
            self.count += 1;
        }
    }
}

impl winit::application::ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_wgpu_ctx.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.winit_wgpu_ctx.window_event(event_loop, _window_id, &event);
        self.ui.window_event(&event, &self.winit_wgpu_ctx.window);

        if event == winit::event::WindowEvent::RedrawRequested {
            if self.ui.needs_update() {
                self.ui.begin_frame();
                self.update_ui();
                self.ui.finish_frame();
            }

            if self.ui.needs_rerender() {
                self.winit_wgpu_ctx.render_ui(&mut self.ui);
            }
        }
                
        if self.ui.event_loop_needs_to_wake() {
            self.winit_wgpu_ctx.window.request_redraw();
        }
    
    }
}
