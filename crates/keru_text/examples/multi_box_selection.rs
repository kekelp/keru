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

        // Create styles with different sizes and colors
        let style_large_white = text.add_style(TextStyle2 {
            font_size: 22.0,
            brush: ColorBrush([255, 255, 255, 255]),
            ..Default::default()
        }, None);

        let style_medium_cyan = text.add_style(TextStyle2 {
            font_size: 18.0,
            brush: ColorBrush([100, 220, 255, 255]),
            ..Default::default()
        }, None);

        let style_medium_yellow = text.add_style(TextStyle2 {
            font_size: 18.0,
            brush: ColorBrush([255, 220, 100, 255]),
            ..Default::default()
        }, None);

        let style_small_green = text.add_style(TextStyle2 {
            font_size: 16.0,
            brush: ColorBrush([150, 255, 150, 255]),
            ..Default::default()
        }, None);

        let style_small_pink = text.add_style(TextStyle2 {
            font_size: 16.0,
            brush: ColorBrush([255, 150, 200, 255]),
            ..Default::default()
        }, None);

        // Create a sequence of linked text boxes that form a paragraph split across multiple boxes
        let box1 = text.add_text_box(
            "This is the first text box. It contains some text that you can select.",
            Some((50.0, 50.0)),
            (400.0, 60.0),
            0.0
        );
        text.get_text_box_mut(&box1).set_style(&style_large_white);

        let box2 = text.add_text_box(
            "This is the second text box, linked after the first. Selection should continue here.",
            Some((50.0, 180.0)),
            (400.0, 60.0),
            0.0
        );
        text.get_text_box_mut(&box2).set_style(&style_medium_cyan);

        let box3 = text.add_text_box(
            "And this is the third text box. The chain continues through all three boxes.",
            Some((50.0, 280.0)),
            (400.0, 60.0),
            0.0
        );
        text.get_text_box_mut(&box3).set_style(&style_medium_yellow);

        // Link the boxes in sequence: box1 -> box2 -> box3
        text.link_text_boxes(&box1, &box2);
        text.link_text_boxes(&box2, &box3);

        // Add a second chain of boxes to test multiple independent chains (horizontal, below main chain)
        let chain2_box1 = text.add_text_box(
            "Second chain, first box. ",
            Some((50.0, 400.0)),
            (250.0, 50.0),
            0.0
        );
        text.get_text_box_mut(&chain2_box1).set_style(&style_small_green);

        let chain2_box2 = text.add_text_box(
            "Second chain, second box. ",
            Some((320.0, 400.0)),
            (250.0, 50.0),
            0.0
        );
        text.get_text_box_mut(&chain2_box2).set_style(&style_small_pink);

        let chain2_box3 = text.add_text_box(
            "Second chain, third box.",
            Some((590.0, 400.0)),
            (250.0, 50.0),
            0.0
        );
        text.get_text_box_mut(&chain2_box3).set_style(&style_small_green);

        text.link_text_boxes(&chain2_box1, &chain2_box2);
        text.link_text_boxes(&chain2_box2, &chain2_box3);

        Self { device, queue, surface, surface_config, window, text, }
    }
}

struct Application { state: Option<State> }

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(event_loop.create_window(
                Window::default_attributes()
                    .with_title("Multi-Box Selection Example")
                    .with_inner_size(winit::dpi::LogicalSize::new(900, 550))
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
