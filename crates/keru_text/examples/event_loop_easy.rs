// This example shows how to integrate the library with a smarter event loop in applications that pause their event loops when nothing is happening.
//
// Usually, this just means checking the result of `Text::handle_event()`, and calling `Window::request_redraw()` only if `result.need_rerender` is true.
// This covers normal updates, as well as smooth scroll animations.
// 
// For cursor blinking, this example uses the auto-wakeup system, where the `Text` struct takes a window Arc reference and spawns a background thread that calls `window.request_redraw()` when the cursor needs to blink.
//
// This is a simple system meant to be as easy as possible to integrate, but it doesn't work for multi-window applications.
//
// See the `event_loop_smart.rs` example to see a more proper implementation of waking.
//
// If you're building an application that never pauses its winit event loop, like a game, you can disregard all the wakeup mechanisms entirely. See the `full.rs` example.


use keru_text::*;
use std::sync::Arc;
use wgpu::*;
use winit::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    
    event_loop
        .run_app(&mut Application { 
            state: None,
        })
        .unwrap();
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,

    text: Text,

    _text_edit: TextEditHandle,
    _text_box: TextBoxHandle,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))
        .unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface_format);
        let new_arc_clone = window.clone();
        text.set_auto_wakeup(new_arc_clone);
        
        let text_edit = text.add_text_edit("This is a text edit box with a bunch of text that can be scrolled. Use the mouse wheel to get a smooth scroll animation. And you can check the console output to see that we're only rerendering when needed.".to_string(), Some((50.0, 50.0)), (400.0, 80.0), 0.0,);
        let text_box = text.add_text_box("This is a regular non-editable text box.", Some((50.0, 180.0)), (500.0, 120.0), 0.0,);

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            text,
            _text_edit: text_edit,
            _text_box: text_box,
        }
    }

    fn render(&mut self) {
        println!("Rerender at {:?}", std::time::Instant::now());
        
        self.text.prepare_all();

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            other => panic!("Failed to get current surface texture: {other:?}"),
        };
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLUE),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            self.text.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(surface_texture);
    }

    fn resize(&mut self, new_size: LogicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

struct Application {
    state: Option<State>,
}

impl winit::application::ApplicationHandler<()> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Smart render loop")
            .with_inner_size(LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window.set_ime_allowed(true);
        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();

        state.text.handle_event(&event, &state.window);

        match &event {
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::Resized(physical_size) => {
                let logical_size = LogicalSize::new(physical_size.width, physical_size.height);
                state.resize(logical_size);
            }
            _ => {}
        }

        if state.text.needs_rerender() {
            state.window.request_redraw();
        }
    }
}
