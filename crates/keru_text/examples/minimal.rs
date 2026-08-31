use std::sync::Arc;
use winit::{event::WindowEvent, event_loop::EventLoop, window::Window};
use wgpu::*;
use keru_text::*;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut Application { state: None }).unwrap();
}

struct State {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    text: Text,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor::default())).unwrap();
        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_config = surface.get_default_config(&adapter, window.inner_size().width, window.inner_size().height).unwrap();
        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface_config.format);
        let text_edit_handle = text.add_text_edit("".to_string(), Some((50.0, 50.0)), (400.0, 200.0), 0.0);

        text.get_text_edit_mut(&text_edit_handle).set_placeholder_static("Type here...");

        Self { device, queue, surface, surface_config, window, text, }
    }
}

struct Application { state: Option<State> }

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(event_loop.create_window(
                Window::default_attributes()
            ).unwrap());
            window.set_ime_allowed(true);
            self.state = Some(State::new(window));
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.text.handle_event(&event, &state.window);
        
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                (state.surface_config.width, state.surface_config.height) = (size.width, size.height);
                state.surface.configure(&state.device, &state.surface_config);
            }
            WindowEvent::RedrawRequested => {
                state.text.prepare_all();

                let surface_texture = match state.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let mut encoder = state.device.create_command_encoder(&CommandEncoderDescriptor::default());
                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &surface_texture.texture.create_view(&TextureViewDescriptor::default()),
                            resolve_target: None,
                            ops: Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });
                    state.text.render(&mut pass);
                }
        
                state.queue.submit(Some(encoder.finish()));
                state.queue.present(surface_texture);

                state.window.request_redraw();
            },
            _ => {}
        }
    }
}
