pub use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
pub use winit::event_loop::EventLoop;
use winit::window::*;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use wgpu::{
    CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::event::WindowEvent;

use keru::Ui;

pub const BACKGROUND_GREY: wgpu::Color = wgpu::Color {
    r: 0.017,
    g: 0.019,
    b: 0.017,
    a: 1.0,
};

pub fn basic_wgpu_init() -> (wgpu::Instance, wgpu::Device, wgpu::Queue) {
    let instance = Instance::new(&InstanceDescriptor {
        ..Default::default()
    });

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        // The color picker draws with a minimal pipeline that takes its rect + parameters as push constants.
        required_features: Features::PUSH_CONSTANTS,
        required_limits: Limits {
            max_push_constant_size: 64,
            ..Limits::defaults()
        },
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
        experimental_features: ExperimentalFeatures::disabled(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc)).unwrap();

    return (instance, device, queue);
}

pub fn basic_surface_config(width: u32, height: u32) -> wgpu::SurfaceConfiguration {
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

pub struct AutoUnwrap<T>(pub Option<T>);
impl<T> Deref for AutoUnwrap<T> {
    type Target = T;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        return self.0.as_ref().unwrap();
    }
}
impl<T> DerefMut for AutoUnwrap<T> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        return self.0.as_mut().unwrap();
    }
}

pub struct Context {
    pub window: AutoUnwrap<Arc<winit::window::Window>>,
    pub surface: AutoUnwrap<wgpu::Surface<'static>>,

    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,

    // Keru used to expose its own projection/screen-size uniform via `Ui::base_uniform_buffer()`.
    // That's gone, so we keep our own equivalent here for the custom canvas shader.
    // Layout matches the `BaseUniforms` struct in canvas.wgsl: `screen_size: vec2f`, `t: f32`.
    pub base_uniform_buffer: wgpu::Buffer,
}

impl Context {
    pub fn new() -> Self {
        // At this point we don't even have a window, so the size here doesn't matter.
        // The correct size will be set on the first resize event.
        let (width, height) = (1920, 1080);

        let (instance, device, queue) = basic_wgpu_init();

        let config = basic_surface_config(width, height);

        let base_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Base Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: 16,
            mapped_at_creation: false,
        });

        let ctx = Self {
            window: AutoUnwrap(None),
            surface: AutoUnwrap(None),
            surface_config: config,
            device,
            queue,
            instance,
            base_uniform_buffer,
        };

        return ctx;
    }

    fn update_base_uniforms(&mut self) {
        let data: [f32; 4] = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
            0.0,
            0.0,
        ];
        self.queue.write_buffer(&self.base_uniform_buffer, 0, bytemuck::bytes_of(&data));
    }

    pub fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let surface = self.instance.create_surface(window.clone()).unwrap();

        self.surface = AutoUnwrap(Some(surface));
        self.window = AutoUnwrap(Some(window));

        self.resize(self.window.inner_size());
    }

    pub fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) {
        let _ = window_id;
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize(*size);
            }
            _ => (),
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.update_base_uniforms();
        self.window.request_redraw();
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn begin_frame(&mut self) -> RenderFrame {
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let surface_texture = self.surface.get_current_texture().unwrap();

        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        return RenderFrame {
            encoder,
            surface_texture,
            view,
        };
    }

    pub fn finish_frame(&mut self, frame: RenderFrame) {
        self.queue.submit(Some(frame.encoder.finish()));
        self.window.pre_present_notify();
        frame.surface_texture.present();
    }

    pub fn render_ui(&mut self, ui: &mut Ui, background_color: wgpu::Color) {
        ui.autorender(&self.surface, background_color);
    }
}

pub struct RenderFrame {
    pub encoder: wgpu::CommandEncoder,
    pub surface_texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}

impl RenderFrame {
    pub fn begin_render_pass(&mut self, bg_color: wgpu::Color) -> wgpu::RenderPass<'_> {
        let color_att = basic_color_attachment(&self.view, bg_color);

        let render_pass_desc = RenderPassDescriptor {
            label: None,
            color_attachments: &color_att,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let render_pass = self.encoder.begin_render_pass(&render_pass_desc);
        return render_pass;
    }
}

pub fn basic_color_attachment(
    view: &wgpu::TextureView,
    bg_color: wgpu::Color,
) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 1] {
    return [Some(RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(bg_color),
            store: wgpu::StoreOp::Store,
        },
        depth_slice: None,
    })];
}
