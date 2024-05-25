use std::sync::Arc;

use wgpu::{
    SurfaceConfiguration, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, TextureFormat, TextureUsages, TextureView
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

pub struct WgpuWindow<'window> {
    pub window: Arc<Window>,
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
}
impl<'window> WgpuWindow<'window> {
    pub fn new(window: Arc<Window>, surface: Surface<'window>, config: SurfaceConfiguration, device: Device, queue: Queue) -> Self {
        return WgpuWindow {
            window,
            surface,
            config,
            device,
            queue,
        };
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
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
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
            load: LoadOp::Load,
            store: wgpu::StoreOp::Store,
        },
    })];
}

pub const ENC_DESC: CommandEncoderDescriptor = CommandEncoderDescriptor { label: None };

pub fn is_redraw_requested(event: &Event<()>) -> bool {
    if let Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } = event {
        return true;
    } else {
        return false;
    }
}