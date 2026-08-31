// This example shows how to handle events in cases where text boxes might be occluded by other objects, using the process described in the crate-level docs.

use keru_text::*;
use std::{sync::Arc, time::Duration};
use wgpu::*;
use winit::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window,
    keyboard::ModifiersState,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut Application { state: None })
        .unwrap();
}

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const OCCLUDING_RECT_DEPTH: f32 = 0.5; // In front of text (text depth is 1.0)

#[allow(dead_code)]
struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,

    text: Text,

    text_edit_handle: TextEditHandle,
    cursor_pos: (f64, f64),
    modifiers: ModifiersState,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor::default())).unwrap();
        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface");
        let surface_config = surface
            .get_default_config(&adapter, physical_size.width, physical_size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface_config.format);
        
        // Create a text edit in the center of the screen (which will be in the occluded right half)
        let text_edit_handle = text.add_text_edit(
            "The right half of this text is occluded.\n\
            I tried rendering something on the right half, but it takes too much code. \n\
            Just imagine that the right half of the screen is occluded by a semitransparent panel.
            ".to_string(),
            Some((200.0, 200.0)), // Center of 800x600 window
            (350.0, 600.0),
            1.0, // Behind the occluding rectangle (depth 0.5)
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            text,
            text_edit_handle,
            cursor_pos: (0.0, 0.0),
            modifiers: ModifiersState::default(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: WindowEvent,
    ) {
        self.handle_event_with_occlusion(&event);

        match event {
            WindowEvent::Resized(size) => {
                self.surface_config.width = size.width;
                self.surface_config.height = size.height;
                self.surface.configure(&self.device, &self.surface_config);
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let view = frame.texture.create_view(&TextureViewDescriptor::default());

                self.text.prepare_all();

                let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.2, a: 1.0, }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });

                    self.text.render(&mut pass);
                }

                self.queue.submit(Some(encoder.finish()));
                self.queue.present(frame);

                std::thread::sleep(Duration::from_millis(16));
                self.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    fn handle_event_with_occlusion(&mut self, event: &WindowEvent) {
        if let WindowEvent::MouseInput { .. } = event {

            // Find which text box would receive the event, and find out if the occluding rectangle would receive the event
            let topmost_text_box_opt = self.text.find_topmost_text_box(event);
            let occluding_rect_hit = self.cursor_pos.0 > WINDOW_WIDTH as f64 / 2.0;
            
            // If no box is hit, or if no other object is hit, there's nothing to compare.
            // We still use handle_event_with_topmost to avoid doing the hit-scanning twice.
            let Some(topmost_text_box) = topmost_text_box_opt else {
                // ... run custom code to handle the rectangle being clicked, if there was any.
                // We still need to call handle_event_with_topmost with None, e.g. to defocus the text box if needed.  
                self.text.handle_event_with_topmost(event, &self.window, None);
                return;
            };
            if ! occluding_rect_hit {
                self.text.handle_event_with_topmost(event, &self.window, Some(topmost_text_box));
                return;
            };

            // If both text and rectangle are hit, compare depths.
            let text_depth = self.text.get_text_box_depth(&topmost_text_box);
            if text_depth < OCCLUDING_RECT_DEPTH {
                self.text.handle_event_with_topmost(event, &self.window, Some(topmost_text_box));
            } else {
                self.text.handle_event_with_topmost(event, &self.window, None);
                // ... and run custom code to handle the rectangle being clicked, if there was any.
            }

        } else {
            self.text.handle_event(event, &self.window);
        }
    }
}

struct Application {
    state: Option<State>,
}

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
            .with_resizable(false)
            .with_title("Occlusion Test - Right half is occluded");
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
        if let Some(state) = &mut self.state {
            state.window_event(event_loop, event);
        };
    }
}