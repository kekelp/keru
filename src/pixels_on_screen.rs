pub use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
pub use winit::{
    error::EventLoopError, event_loop::EventLoop, event::Event, event_loop::EventLoopWindowTarget
};

use std::sync::Arc;

use wgpu::{
    Color, CommandEncoder, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, TextureView
};
use winit::{
    dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, window::{Window as WinitWindow, WindowBuilder}
};

pub fn basic_wgpu_init() -> (Instance, Device, Queue) {
    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        // Downlevel defaults are really bad. Maximum texture size = 2048 means you can't even maximize a window on a 1440p screen.
        required_limits: Limits::default(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    return (instance, device, queue);
}

pub fn basic_surface_config(width: u32, height: u32) -> SurfaceConfiguration {
    return SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width,
        height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
}

pub struct Context {
    pub window: Arc<WinitWindow>,
    pub surface: Surface<'static>,
    
    pub surface_config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
}
impl Context {
    pub fn init(width: u32, height: u32, title: &str) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            WindowBuilder::new()
                .with_inner_size(LogicalSize::new(width, height))
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );
    
        let (instance, device, queue) = basic_wgpu_init();

        let surface = instance.create_surface(window.clone()).unwrap();

        let config = basic_surface_config(width, height);
        surface.configure(&device, &config);

        let ctx = Self {
            window,
            surface,
            surface_config: config,
            device,
            queue,
        };

        return (ctx, event_loop);
    }

    pub fn handle_events(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {        
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.resize(size),
            Event::AboutToWait => {
                self.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            _ => {}
        }
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.window.request_redraw();
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn begin_frame(&mut self) -> RenderFrame {        
        let encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        return RenderFrame {
            encoder,
            frame,
            view
        };
    }
}

pub struct RenderFrame {
    pub encoder: CommandEncoder,
    pub frame: SurfaceTexture,
    pub view: TextureView,
}

impl RenderFrame {
    pub fn begin_render_pass(&mut self, bg_color: Color) -> RenderPass<'_> {
        let color_att = basic_color_attachment(&self.view, bg_color);
        let render_pass_desc = &basic_render_pass_desc(&color_att);
        let render_pass = self.encoder.begin_render_pass(render_pass_desc);
        return render_pass;
    }

    pub fn finish(self, queue: &Queue) {
        queue.submit(Some(self.encoder.finish()));
        self.frame.present();
    }
}

pub fn basic_render_pass_desc<'a>(
    color_att: &'a [Option<RenderPassColorAttachment<'a>>; 1],
) -> RenderPassDescriptor<'a> {
    return RenderPassDescriptor {
        label: None,
        color_attachments: color_att,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    };
}

pub fn basic_color_attachment(view: &TextureView, bg_color: Color) -> [Option<RenderPassColorAttachment<'_>>; 1] {
    return [Some(RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(bg_color),
            store: wgpu::StoreOp::Store,
        },
    })];
}

pub fn is_redraw_requested(event: &Event<()>) -> bool {
    if let Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } = event {
        return true;
    } else {
        return false;
    }
}

pub trait Scale {
    fn scale(self, scale: f32) -> Self;
}
impl Scale for usize {
    fn scale(self, scale: f32) -> Self {
        return (self as f32 * scale) as Self;
    }
}
impl Scale for u32 {
    fn scale(self, scale: f32) -> Self {
        return (self as f32 * scale) as Self;
    }
}