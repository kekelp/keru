use std::sync::Arc;


use wgpu::{
    CompositeAlphaMode, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PresentMode, RequestAdapterOptions, TextureUsages, Surface, SurfaceConfiguration, Device, Queue, RenderPassDescriptor, RenderPassColorAttachment, TextureView, Operations, LoadOp, CommandEncoderDescriptor,
};
use winit::{
    dpi::{LogicalSize},
    event_loop::{EventLoop},
    window::{Window, WindowBuilder},
};

use crate::SWAPCHAIN_FORMAT;


pub fn init_winit_window(width: f64, height: f64) -> ( EventLoop<()>, Arc<Window> ) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("BLUE")
            .build(&event_loop)
            .unwrap(),
    );

    return (event_loop, window);
}



pub fn init_wgpu_renderer(window: &Window) -> ( Surface, SurfaceConfiguration, Device, Queue, ) {

    let size = window.inner_size();

    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::default(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    let surface = instance.create_surface(window).unwrap();

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

    return (surface, config, device, queue);
}

pub fn base_render_pass_desc<'tex, 'desc>(color_att: &'desc [Option<RenderPassColorAttachment<'tex>>; 1]) -> RenderPassDescriptor<'tex, 'desc> {
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
    })]
}

pub const ENC_DESC: CommandEncoderDescriptor = CommandEncoderDescriptor { label: None };