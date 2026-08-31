use std::sync::Arc;
use winit::{event::WindowEvent, event_loop::EventLoop, keyboard::ModifiersState, window::Window};
use wgpu::*;
use keru_text::*;
use parley::*;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut Application { state: None }).unwrap();
}

const MONOSPACE: StyleProperty<ColorBrush> = StyleProperty::FontFamily(FontFamily::Single(FontFamilyName::Generic(GenericFamily::Monospace)));
const BOLD: StyleProperty<ColorBrush> = StyleProperty::FontWeight(FontWeight::new(600.0));
const ITALIC: StyleProperty<ColorBrush> = StyleProperty::FontStyle(FontStyle::Italic);
const RED: StyleProperty<ColorBrush> = StyleProperty::Brush(ColorBrush([255, 80, 80, 255]));
const GREEN: StyleProperty<ColorBrush> = StyleProperty::Brush(ColorBrush([80, 255, 80, 255]));
const CERULEAN: StyleProperty<ColorBrush> = StyleProperty::Brush(ColorBrush([50, 110, 255, 255]));
const LARGE: StyleProperty<ColorBrush> = StyleProperty::FontSize(28.0);
const SMALL: StyleProperty<ColorBrush> = StyleProperty::FontSize(16.0);

struct State {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    text: Text,
    edit_handle: TextEditHandle,
    modifiers: winit::keyboard::ModifiersState,
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

        let initial = "Lorem ipsum dolor sit amet, qui summo legere nusquam ad, eu mel doming essent deseruisse. Nec nulla nostrum disputationi cu. Ut vim sadipscing voluptatibus, vis evertitur dissentiunt ad. Nonumy graeco noluisse id duo, sea reque omnesque insolens ea.

        Vide modus et sed. Has an nullam facete disputando, eum at case volumus officiis. Cum cu magna graeco mandamus, no purto erat eruditi sit. Et alia tractatos nam, soleat eruditi ne pri. Quo at nullam nusquam dissentiunt. Primis quodsi per no. Pri choro ubique ei, ut sit oporteat consetetur.";

        let edit_handle = text.add_text_edit(initial.to_string(), Some((50.0, 50.0)), (500.0, 200.0), 0.0);

        let edit = text.get_text_edit_mut(&edit_handle);

        // Using handwritten ranges like this is only safe is the text is basic ASCII.
        edit.push_style_property(RED, 0..15);
        edit.push_style_property(SMALL, 0..15);

        edit.push_style_property(GREEN, 33..66);

        edit.push_style_property(CERULEAN, 81..126);
        edit.push_style_property(LARGE, 81..126);

        let modifiers = ModifiersState::default();

        let info = "\
            Use keyboard shortcuts to change text properties of the selected text in the edit box above. \n\n\
            Ctrl + M: Make Monospaced \n\
            Ctrl + B: Make Bold \n\
            Ctrl + I: Make Italic \n\
            Ctrl + L: Make Large \n\
            Ctrl + S: Make Small \n\
            Ctrl + R: Make Red \n\
            Ctrl + G: Make Green \n\
            Ctrl + C: Make Cerulean \n\
            Ctrl + N: Clear all properties and return to normal \n\
        ";
        let info_box = text.add_text_box(info, Some((50.0, 300.0)), (500.0, 250.0), 0.0);
        text.get_text_box_mut(&info_box).push_ranged_style_property(SMALL, 0..info.len());

        Self { device, queue, surface, surface_config, window, text, edit_handle, modifiers }
    }
}

struct Application { state: Option<State> }

impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(event_loop.create_window(
                Window::default_attributes().with_title("Ranged styles")
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
            WindowEvent::ModifiersChanged(modifiers) => {
                state.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed && state.modifiers.control_key() {
                    let text_edit = state.text.get_text_edit_mut(&state.edit_handle);
                    if let Some(s) = event.text && let Some(range) = text_edit.selected_text_range() {
                        
                        match s.as_str() {
                            "m" => {
                                text_edit.push_style_property(MONOSPACE, range.clone());
                            },
                            "b" => {
                                text_edit.push_style_property(BOLD, range.clone());
                            },
                            "i" => {
                                text_edit.push_style_property(ITALIC, range.clone());
                            },
                            "l" => {
                                text_edit.push_style_property(LARGE, range.clone());
                            },
                            "s" => {
                                text_edit.push_style_property(SMALL, range.clone());
                            },
                            "c" => {
                                text_edit.push_style_property(CERULEAN, range.clone());
                            },
                            "r" => {
                                text_edit.push_style_property(RED, range.clone());
                            },
                            "g" => {
                                text_edit.push_style_property(GREEN, range.clone());
                            },
                            "n" => {
                                text_edit.clear_style_properties_in_range(range.clone());
                            },
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
