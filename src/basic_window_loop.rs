//! Helper functions for `winit` and `wgpu`.
pub use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
pub use winit::{error::EventLoopError, event::Event, event_loop::EventLoop};
use winit::{event_loop::ActiveEventLoop, window::*};

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use wgpu::{
    Color, CommandEncoder, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat,
    TextureUsages, TextureView,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window as WinitWindow};

use crate::Ui;

pub const BACKGROUND_GREY: wgpu::Color = wgpu::Color {
    r: 0.017,
    g: 0.019,
    b: 0.017,
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
    };
}

pub struct AutoUnwrap<T>(Option<T>);
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
    // Winit wants us to initialize all our stuff before it actually creates a window for us, so the best we can do is creating something like an Option, set it to None initially, and then put the window there when we finally get it.
    // This sort of makes sense: during its lifetime a process can have a variable number of window ranging from zero to multiple. So in the general case you'd have something like a Vec of windows that can be empty. An Option is a simpler version of that.
    // But for most programs including this basic example, none of this is relevant, there's only one window that lasts for the whole duration of the program, and even having to unwrap an Option every time we want to use it is just annoying for no good reason.
    // Luckily, we can avoid that with this AutoUnwrap struct. Normally this would be a very weird thing to do, but Winit's loop is weird enough that it makes sense here.
    //
    // The Arc is needed for even more arcane reasons.
    pub window: AutoUnwrap<Arc<WinitWindow>>,
    pub surface: AutoUnwrap<Surface<'static>>,

    pub surface_config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    pub instance: Instance,

    pub depth_stencil_texture: Texture,
}

impl Context {
    pub fn new() -> Self {
        // At this point we don't even have a window, so it doesn't matter what we write here.
        // Again, the winit loop is a bit weird.
        // The correct size will be set on the first resize event.
        let (width, height) = (1920, 1080);

        let (instance, device, queue) = basic_wgpu_init();

        let config = basic_surface_config(width, height);

        let depth_tex_desc = basic_depth_texture_descriptor(width, height);
        let depth_stencil_texture = device.create_texture(&depth_tex_desc);

        let ctx = Self {
            window: AutoUnwrap(None),
            surface: AutoUnwrap(None),
            surface_config: config,
            device,
            queue,
            depth_stencil_texture,
            instance,
        };

        return ctx;
    }

    pub fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let surface = self.instance.create_surface(window.clone()).unwrap();

        self.surface = AutoUnwrap(Some(surface));
        self.window = AutoUnwrap(Some(window));
        self.window.set_ime_allowed(true);

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

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
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
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let surface_texture = self.surface.get_current_texture().unwrap();

        let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());
        
        // maybe we could avoid recreating this view every time.
        let depth_stencil_view = self
            .depth_stencil_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        return RenderFrame {
            encoder,
            surface_texture,
            view,
            depth_stencil_view,
        };
    }

    pub fn finish_frame(&mut self, frame: RenderFrame) {
        self.queue.submit(Some(frame.encoder.finish()));
        self.window.pre_present_notify();
        frame.surface_texture.present();
    }

    pub fn render_ui(&mut self, ui: &mut Ui) {
        let mut frame = self.begin_frame();

        // Without these brackets, wgpu panics. I think this used to be a compiler error in older versions? 
        {
            let mut render_pass = frame.begin_render_pass(BACKGROUND_GREY);
            ui.render(&mut render_pass, &self.device, &self.queue);
        }
        
        self.finish_frame(frame);
    }
}

pub struct RenderFrame {
    pub encoder: CommandEncoder,
    pub surface_texture: SurfaceTexture,
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
}

pub fn basic_color_attachment(
    view: &TextureView,
    bg_color: Color,
) -> [Option<RenderPassColorAttachment<'_>>; 1] {
    return [Some(RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(bg_color),
            store: wgpu::StoreOp::Store,
        },
    })];
}

pub fn basic_depth_stencil_attachment(
    depth_stencil_view: &TextureView,
) -> Option<RenderPassDepthStencilAttachment<'_>> {
    return Some(wgpu::RenderPassDepthStencilAttachment {
        view: depth_stencil_view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
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
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = self
        {
            return true;
        } else {
            return false;
        }
    }
}

pub fn basic_env_logger_init() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::", log::LevelFilter::Info)
        .init();
}
