pub use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};

use std::sync::Arc;

use winit_input_helper::WinitInputHelper;

use wgpu::{
    Color, CommandEncoder, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, TextureView
};
use winit::{
    dpi::{LogicalSize, PhysicalSize}, event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopWindowTarget}, window::{Window, WindowBuilder}
};

pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
pub fn configure_surface(surface: &Surface, window: &Window, device: &Device) -> SurfaceConfiguration {
    let size = window.inner_size();
    let config = base_surface_config(size.width, size.height, SWAPCHAIN_FORMAT);
    surface.configure(&device, &config);

    return config;
}

pub struct Context {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub input: WinitInputHelper,
    
    pub surface_config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
}
impl Context {
    pub fn new(window: Arc<Window>, surface: Surface<'static>, config: SurfaceConfiguration, device: Device, queue: Queue) -> Self {
        return Context {
            window,
            surface,
            surface_config: config,
            device,
            queue,
            input: WinitInputHelper::new(),
        };
    }

    pub fn handle_events(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {

        self.input.update(&event);
        
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
    pub fn begin_render_pass<'pass>(&'pass mut self) -> RenderPass<'pass> {
        let color_att = base_color_attachment(&self.view);
        let render_pass_desc = &base_render_pass_desc(&color_att);
        let render_pass = self.encoder.begin_render_pass(render_pass_desc);
        return render_pass;
    }

    pub fn finish(self, queue: &Queue) {
        queue.submit(Some(self.encoder.finish()));
        self.frame.present();
    }
}

pub fn init_winit_and_wgpu(width: f64, height: f64) -> (EventLoop<()>, Arc<Window>, Instance, Device, Queue) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("BLUE")
            .build(&event_loop)
            .unwrap(),
    );

    let (instance, device, queue) = init_wgpu();

    return (event_loop, window, instance, device, queue);
}

pub fn init_wgpu() -> (Instance, Device, Queue) {
    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::default(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    return (instance, device, queue);
}

pub fn base_surface_config(width: u32, height: u32, format: TextureFormat) -> SurfaceConfiguration {
    return SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width,
        height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
}

pub fn base_render_pass_desc<'tex, 'desc>(
    color_att: &'desc [Option<RenderPassColorAttachment<'tex>>; 1],
) -> RenderPassDescriptor<'tex, 'desc> {
    return RenderPassDescriptor {
        label: None,
        color_attachments: color_att,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    };
}

pub fn base_color_attachment(view: &TextureView) -> [Option<RenderPassColorAttachment<'_>>; 1] {
    return [Some(RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(Color { r: 0.0014, g: 0.0014, b: 0.015, a: 1.0 }),
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