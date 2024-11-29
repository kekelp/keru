pub use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
pub use winit::{
    error::EventLoopError, event_loop::EventLoop, event::Event, event_loop::EventLoopWindowTarget
};

use core::f32;
use std::{sync::Arc, thread, time::{Duration, Instant}};

use wgpu::{
    Color, CommandEncoder, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat, TextureUsages, TextureView
};
use winit::{
    dpi::{LogicalSize, PhysicalSize}, event::WindowEvent, window::{Window as WinitWindow, WindowBuilder}
};

pub const BACKGROUND_GREY: wgpu::Color = wgpu::Color {
    r: 0.037,
    g: 0.039,
    b: 0.037,
    a: 1.0,
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

pub fn basic_depth_texture_descriptor(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    return wgpu::TextureDescriptor {
        label: Some("Depth Stencil"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }
}

pub struct Context {
    pub window: Arc<WinitWindow>,
    pub surface: Surface<'static>,
    
    pub surface_config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,

    pub depth_stencil_texture: Texture,

    pub last_frame_timestamp: Instant,
    pub current_frame_timestamp: Instant,
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

        let depth_tex_desc = basic_depth_texture_descriptor(width, height);
        let depth_stencil_texture = device.create_texture(&depth_tex_desc);

        let ctx = Self {
            window,
            surface,
            surface_config: config,
            device,
            queue,
            depth_stencil_texture,
            last_frame_timestamp: Instant::now(),
            current_frame_timestamp: Instant::now(),
        };

        return (ctx, event_loop);
    }

    pub fn handle_events(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {        
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => self.resize(size),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => target.exit(),
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                self.last_frame_timestamp = self.current_frame_timestamp;
                self.current_frame_timestamp = Instant::now();
                self.window.request_redraw();
            },
            _ => {}
        }
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        let depth_tex_desc = basic_depth_texture_descriptor(size.width, size.height);
        self.depth_stencil_texture = self.device.create_texture(&depth_tex_desc);

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

        // todo: why recreate the views on every frame
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let depth_stencil_view = self.depth_stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());

        return RenderFrame {
            encoder,
            frame,
            view,
            depth_stencil_view,
        };
    }

    pub fn sleep_until_next_frame(&mut self) {
        let refresh_rate = self.window.current_monitor().unwrap().video_modes().next().unwrap().refresh_rate_millihertz();        
        let frame_time_micros = (1_000_000_000 / refresh_rate) as u64;
        let sleep_time = Duration::from_micros(frame_time_micros);

        thread::sleep(sleep_time);
    }
}

pub struct RenderFrame {
    pub encoder: CommandEncoder,
    pub frame: SurfaceTexture,
    pub view: TextureView,
    pub depth_stencil_view: TextureView,
}

impl RenderFrame {
    pub fn begin_render_pass(&mut self, bg_color: Color) -> RenderPass<'_> {
        let color_att = basic_color_attachment(&self.view, bg_color);
        let depth_att = basic_depth_stencil_attachment(&self.depth_stencil_view);
        
        let render_pass_desc = RenderPassDescriptor {
            label: None,
            color_attachments: &color_att,
            depth_stencil_attachment: depth_att,
            // depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let render_pass = self.encoder.begin_render_pass(&render_pass_desc);
        return render_pass;
    }

    pub fn finish(self, queue: &Queue) {
        queue.submit(Some(self.encoder.finish()));
        self.frame.present();
    }
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

pub fn basic_depth_stencil_attachment(depth_stencil_view: &TextureView) -> Option<RenderPassDepthStencilAttachment<'_>> {
    return Some(wgpu::RenderPassDepthStencilAttachment {
        view: &depth_stencil_view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(f32::MAX),
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
    });
}

pub fn basic_depth_stencil_state() -> wgpu::DepthStencilState {
    return wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    };
}

pub trait EventIsRedrawRequested {
    fn is_redraw_requested(&self) -> bool;
}
impl EventIsRedrawRequested for Event<()> {
    fn is_redraw_requested(&self) -> bool {
        if let Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } = self {
            return true;
        } else {
            return false;
        }
    }
}
