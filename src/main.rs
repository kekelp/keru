pub mod render;
pub mod ui;
pub use ui::Id;

use glyphon::Resolution as GlyphonResolution;
use ui::{floating_window_1, Color, LayoutMode, NodeKey, Rectangle, Ui};
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use std::sync::Arc;

use crate::ui::NODE_ROOT_ID;

#[rustfmt::skip]
fn main() {
    let (event_loop, mut state) = init();

    event_loop.run(
        move |event, target| {
            state.handle_event(&event, target);
        }
    ).unwrap();
}

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 800;
pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

fn init() -> (EventLoop<()>, State<'static>) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .with_title("BLUE")
            .build(&event_loop)
            .unwrap(),
    );
    let size = window.inner_size();
    // let scale_factor = window.scale_factor();

    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::default(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    let surface = instance.create_surface(window.clone()).unwrap();

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: SWAPCHAIN_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let ui = Ui::new(&device, &config, &queue);

    let state = State {
        window,
        surface,
        config,
        device,
        queue,

        ui,

        // app state
        count: 0,
        counter_mode: true,
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: Arc<Window>,

    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,

    pub ui: Ui,

    pub count: i32,
    pub counter_mode: bool,
}

pub const INCREASE_BUTTON: NodeKey = NodeKey {
    id: id!(),
    static_text: Some("Increase"),
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.1,
        b: 0.1,
        a: 0.9,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 100,
        len: 100,
    },
    is_update: false,
    is_layout_update: false,
};

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey {
    id: id!(),
    static_text: Some("Show counter"),
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 400,
        len: 100,
    },
    is_update: false,
    is_layout_update: false,
};

pub const COUNT_LABEL: NodeKey = NodeKey {
    id: id!(),
    static_text: None,
    dyn_text: None,
    clickable: false,
    color: Color {
        r: 0.1,
        g: 0.3,
        b: 0.9,
        a: 0.6,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.2,
        end: 0.5,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.2,
        end: 0.8,
    },
    is_update: false,
    is_layout_update: false,
};

pub const COLUMN_1: NodeKey = NodeKey {
    id: id!(),
    static_text: None,
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.2,
        b: 0.7,
        a: 0.2,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.7,
        end: 0.9,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.0,
        end: 1.0,
    },
    is_update: false,
    is_layout_update: false,
};

pub const FLOATING_WINDOW_1: NodeKey = floating_window_1();

impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        self.ui.handle_event(event, &self.queue);

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::RedrawRequested => {
                    self.update();

                    self.window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::CursorMoved { position, .. } => {
                    self.ui.mouse_pos.x = position.x as f32;
                    self.ui.mouse_pos.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        if *state == ElementState::Pressed {
                            self.ui.mouse_left_clicked = true;
                            if !self.ui.mouse_left_just_clicked {
                                self.ui.mouse_left_just_clicked = true;
                            }
                        } else {
                            self.ui.mouse_left_clicked = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        self.ui
            .nodes
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();

        let ui = &mut self.ui;

        div!((ui, FLOATING_WINDOW_1) {

            div!(ui, COUNT_LABEL.with_text(self.count));

            div!((ui, COLUMN_1) {

                let text = match self.counter_mode {
                    true => &"Hide counter",
                    false => &"Show counter",
                };
                div!(ui, SHOW_COUNTER_BUTTON.with_text(text));

                if self.counter_mode {
                    let color = Color { r: 0.1 * (self.count as f32), g: 0.0, b: 0.0, a: 1.0 };
                    div!(ui, INCREASE_BUTTON.with_color(color));
                }

            });

        });

        self.ui.layout();
        // self.resolve_input();

        if self.ui.is_clicked(INCREASE_BUTTON) {
            self.count += 1;
        }

        if self.ui.is_clicked(SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
        }

        self.ui.build_buffers();

        self.render();

        self.ui.current_frame += 1;
        self.ui.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.ui
            .gpu_vertex_buffer
            .queue_write(&self.ui.rects[..], &self.queue);

        self.ui
            .text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.ui.font_system,
                &mut self.ui.atlas,
                GlyphonResolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                &mut self.ui.text_areas,
                &mut self.ui.cache,
                self.ui.current_frame,
            )
            .unwrap();

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let n = self.ui.rects.len() as u32;
            if n > 0 {
                pass.set_pipeline(&self.ui.render_pipeline);
                pass.set_bind_group(0, &self.ui.bind_group, &[]);
                pass.set_vertex_buffer(0, self.ui.gpu_vertex_buffer.slice(n));
                pass.draw(0..6, 0..n);
            }

            self.ui
                .text_renderer
                .render(&self.ui.atlas, &mut pass)
                .unwrap();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.ui.atlas.trim();
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        self.window.request_redraw();
    }
}
