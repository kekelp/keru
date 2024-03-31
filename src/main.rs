use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer,
};
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use std::sync::Arc;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let (width, height) = (800, 600);
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_title("glyphon hello world")
            .build(&event_loop)
            .unwrap(),
    );
    let size = window.inner_size();
    let scale_factor = window.scale_factor();

    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let surface = instance
        .create_surface(window.clone())
        .expect("Create surface");
    let swapchain_format = TextureFormat::Bgra8UnormSrgb;
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Set up text renderer
    let mut font_system = FontSystem::new();
    let cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
    let text_renderer = TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

    let physical_width = (width as f64 * scale_factor) as f32;
    let physical_height = (height as f64 * scale_factor) as f32;

    buffer.set_size(&mut font_system, physical_width, physical_height);
    buffer.set_text(&mut font_system, "Hello world! 👋この動画の元になった作品ヽ༼ ຈل͜ຈ༽ ﾉヽ༼ ຈل͜ຈ༽ ﾉ\nヽ༼ ຈل͜ຈ༽\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system);

    let mut state = State {
        window,
        surface,
        config,
        device,
        cache,
        queue,
        atlas,
        text_renderer,
        font_system,
        buffer,
    };

    #[rustfmt::skip]
    event_loop.run(
        move |event, target| {
            state.handle_event(&event, target);
        }
    ).unwrap();
}

pub struct State<'window> {
    window: Arc<Window>,
    surface: Surface<'window>,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    buffer: Buffer,
}

impl<'a> State<'a> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => {
                    self.config.width = size.width;
                    self.config.height = size.height;
                    self.surface.configure(&self.device, &self.config);
                    self.window.request_redraw();
                }
                WindowEvent::RedrawRequested => self.update_and_render(),
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }
    }

    pub fn update_and_render(&mut self) {
        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                [TextArea {
                    buffer: &self.buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 900,
                        bottom: 660,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                &mut self.cache,
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
                        load: LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer.render(&self.atlas, &mut pass).unwrap();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.atlas.trim();
    }
}
