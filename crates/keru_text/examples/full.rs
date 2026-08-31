use keru_text::*;
use keru_text::parley::*;
use std::{f32::consts::PI, sync::Arc};
use wgpu::*;
use winit::{
    dpi::LogicalSize,
    event::{WindowEvent, ElementState},
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

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,
    window: Arc<Window>,

    text: Text,

    single_line_input: TextEditHandle,

    big_text_style: StyleHandle,
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

        let white = [255,0,0,255];
        let mut text = Text::new(&device, &queue, surface_config.format);
        
        // Create a style
        let big_text_style_handle = text.add_style(TextStyle {
            font_size: 64.0,
            brush: ColorBrush(white),
            font_family: FontFamily::named("sans-serif"),
            overflow_wrap: OverflowWrap::Anywhere,
            ..Default::default()
        }, None);

        // Load a custom font using the `load_font` helper. For more advanced usage, you can access the `parley` FontContext directly via text.font_context().
        let font_name = text.load_font(include_bytes!("PublicPixel.ttf")).expect("Failed to load font");
        // font_name will be "Public Pixel".

        let custom_font_style_handle = text.add_style(TextStyle {
            font_size: 22.0,
            brush: ColorBrush([0, 255, 150, 255]),
            font_family: FontFamily::Single(FontFamilyName::Named(font_name.into())),
            overflow_wrap: OverflowWrap::Anywhere,
            ..Default::default()
        }, None);

        let monospace_style_handle = text.add_style(TextStyle {
            font_size: 22.0,
            font_family: FontFamily::Source("monospace".into()),
            brush: ColorBrush([30, 60, 255, 255]),
            overflow_wrap: OverflowWrap::Anywhere,
            ..Default::default()
        }, None);

        // Create text boxes and get handles. Normally, the handle would be owned by a higher level struct representing a node in a GUI tree or something similar.
        let single_line_input = text.add_text_edit("".to_string(), Some((10.0, 15.0)), (200.0, 35.0), 0.0);
        let editable_text_with_unicode = text.add_text_edit("Editable 🌈 text ⚡⚡ 無限での座を含む全ての سلام دنیا، این یک متن قابل ویرایش است Editable text 無限での座を含む全ての سلام دنیا، این یک متن قابل ویرایش است".to_string(), Some((300.0, 200.0)), (400.0, 200.0), 0.0);
        let _info = text.add_text_box("Press Ctrl + D to disable the top edit box.".to_string(), Some((10.0, 60.0)), (200.0, 100.0), 0.0);
        let _help_text_edit = text.add_text_edit("Press Ctrl + Plus and Ctrl + Minus to adjust the size of the big text.".to_string(), Some((470.0, 60.0)), (200.0, 150.0), 0.0);
        let shift_enter_text_edit = text.add_text_edit("Use Shift+Enter for newlines here".to_string(), Some((250.0, 60.0)), (200.0, 100.0), 0.0);

        let clipped_text_box = text.add_text_box("Clipped text".to_string(), Some((0.0, 0.0)), (1000.0, 1000.0), 0.0);

        // Using a &'static str here for this non-editable text box.
        let justified_static_text = text.add_text_box("Long static words, Long static words, Long static words, Long static words, ... (justified btw) ", Some((150.0, 440.0)), (600.0, 150.0), 0.0);
        
        // Use the handles to access and edit text boxes. Accessing a box through a handle is a very fast operation, basically just an array access. There is no hashing involved.
        text.get_text_edit_mut(&single_line_input).set_single_line(true);
        text.get_text_edit_mut(&single_line_input).set_placeholder_static("Single line input");
        text.get_text_edit_mut(&editable_text_with_unicode).set_style(&big_text_style_handle);
        text.get_text_edit_mut(&shift_enter_text_edit).set_newline_mode(NewlineMode::ShiftEnter);
        text.get_text_edit_mut(&_help_text_edit).set_style(&monospace_style_handle);
        
        text.get_text_box_mut(&clipped_text_box).set_style(&big_text_style_handle);
        text.get_text_box_mut(&clipped_text_box).set_selectable(true);
        text.get_text_box_mut(&clipped_text_box).set_auto_clip(true);
        text.get_text_box_mut(&clipped_text_box).set_clip_rect(Some(BoundingBox {
            x0: 20.0, y0: 200.0, x1: 50.0, y1: 430.0,
        }));

        text.get_text_style_mut(&big_text_style_handle).font_size = 32.0;

        text.get_text_box_mut(&justified_static_text).set_style(&custom_font_style_handle);
        text.get_text_box_mut(&justified_static_text).set_alignment(Alignment::Justify);

        text.get_text_box_mut(&clipped_text_box).set_transform(
            Transform2D {
                translation: (20.0, 400.0),
                rotation: -PI * 0.5,
                scale: 1.0,
            }
        );


        Self {
            device,
            queue,
            surface,
            surface_config,
            window,
            text,

            single_line_input,
            big_text_style: big_text_style_handle,

            modifiers: ModifiersState::default(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: WindowEvent,
    ) {
        self.text.handle_event(&event, &self.window);

        match event {
            WindowEvent::Resized(size) => {
                self.surface_config.width = size.width;
                self.surface_config.height = size.height;
                self.surface.configure(&self.device, &self.surface_config);
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // Prepare the cpu-side text data for rendering.
                self.text.prepare_all();

                // A bunch of wgpu boilerplate to be able to draw on the screen.
                let surface_texture = match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());
                let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });

                    // Finally, render the text on the screen.
                    self.text.render(&mut pass);
                }

                // More boilerplate.
                self.queue.submit(Some(encoder.finish()));
                self.queue.present(surface_texture);
                self.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && self.modifiers.control_key() {
                    if let Some(s) = event.text {
                        match s.as_str() {
                            "+" => {
                                self.text.get_text_style_mut(&self.big_text_style).font_size += 2.0;
                            }
                            "-" => {
                                let current_size = self.text.get_text_style(&self.big_text_style).font_size;
                                if current_size > 4.0 {
                                    self.text.get_text_style_mut(&self.big_text_style).font_size -= 2.0;
                                }
                            }
                            "d" => {
                                let is_disabled = self.text.get_text_edit(&self.single_line_input).disabled();
                                self.text.set_text_edit_disabled(&self.single_line_input, !is_disabled);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
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

        let (width, height) = (800, 600);
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_title("hello world");
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