//! Because Keru doesn't steal control of your rendering loop and thanks to the RenderCommand system, it's relatively easy to implement liquid glass and similar render effects at user level.
//! 
//! This also means that you cna experiment with your own effects rather than hoping that the library remembered to add hardcoded support for the flavor of the month effect.
//! 
//! As in `custom_rendering.rs`, we do have to write a lot of winit and wgpu boilerplate, but it's still not that much code.

use keru::*;
use keru::node_library::*;
use keru::Color;
use wgpu::*;
use std::sync::Arc;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}};

struct Application {
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    config: SurfaceConfiguration,
    ui: Ui,
    glass_pos: [(f32, f32); 2],
    bind_group_layout: BindGroupLayout,
    sampler: Sampler,
    backdrop: Texture,
    backdrop_bind_group: BindGroup,
    glass_pipeline: RenderPipeline,
}

#[node_key] const GLASS_1: NodeKey;
#[node_key] const GLASS_2: NodeKey;

const GLASS_CORNER_RADIUS: f32 = 40.0;

impl State {
    fn new(window: Arc<Window>, instance: Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        // Note that immediate data / push constants are not guaranteed to be supported everywhere. It's just for the example.
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::IMMEDIATES,
            required_limits: Limits { max_immediate_size: 32, ..Default::default() },
            ..Default::default()
        })).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| ! f.is_srgb())
            .copied().unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            // COPY_SRC lets us copy the surface texture into the backdrop texture.
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &config);

        let ui = Ui::new(&device, &queue, &config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Liquid glass shader"),
            source: ShaderSource::Wgsl(include_str!("liquid_glass.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Liquid glass bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Liquid glass sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Liquid glass pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 32,
        });

        let glass_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Liquid glass pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_glass"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_glass"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let (backdrop, backdrop_bind_group) = create_backdrop(&device, &config, &bind_group_layout, &sampler);

        let glass_pos = [(120.0, 140.0), (480.0, 320.0)];

        Self { window, surface, device, config, ui, glass_pos, bind_group_layout, sampler, backdrop, backdrop_bind_group, glass_pipeline }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        (self.backdrop, self.backdrop_bind_group) = create_backdrop(&self.device, &self.config, &self.bind_group_layout, &self.sampler);
    }

    fn update_ui(&mut self) {
        // Colorful background content, so that the refraction is easy to see.
        let background = PANEL
            .color(Color::from_hex_str("#1b1d2b"))
            .size_symm(Size::Frac(1.0))
            .absorbs_clicks(false);

        let blob = |color: &str, x: f32, y: f32, size: f32| {
            DEFAULT
                .shape(Shape::Circle)
                .color(Color::from_hex_str(color))
                .anchor_symm(Anchor::Center)
                .position_x(Pos::Frac(x))
                .position_y(Pos::Frac(y))
                .size_symm(Size::Pixels(size))
                .absorbs_clicks(false)
        };

        let stripe = |i: usize| {
            let colors = ["#ff5e5b", "#ffa166", "#06d6a0", "#118ab2", "#9b5de5"];
            DEFAULT
                .color(Color::from_hex_str(colors[i % colors.len()]))
                .position_x(Pos::Pixels(i as f32 * 70.0))
                .position_y(Pos::Frac(0.7))
                .size_x(Size::Pixels(35.0))
                .size_y(Size::Pixels(140.0))
                .absorbs_clicks(false)
        };

        let big_text = TEXT
            .static_text("User-level liquid glass effect.")
            .position_x(Pos::Pixels(40.0))
            .position_y(Pos::Pixels(40.0))
            .absorbs_clicks(false);

        self.ui.add(background).nest(|| {
            self.ui.add(blob("#ff5e5b", 0.2, 0.35, 260.0));
            self.ui.add(blob("#66c4ff", 0.55, 0.25, 200.0));
            self.ui.add(blob("#06d6a0", 0.75, 0.6, 300.0));
            self.ui.add(blob("#ffa166", 0.35, 0.8, 220.0));
            for i in 0..30 {
                self.ui.add(stripe(i));
            }
            self.ui.add(big_text);
        });

        for (i, key) in [GLASS_1, GLASS_2].into_iter().enumerate() {
            let glass = DEFAULT
                .invisible()
                .custom_render(true)
                .size_x(Size::Pixels(360.0))
                .size_y(Size::Pixels(220.0))
                .position_x(Pos::Pixels(self.glass_pos[i].0))
                .position_y(Pos::Pixels(self.glass_pos[i].1))
                .padding(30.0)
                .sense_drag(true)
                .key(key);

            self.ui.add(glass).nest(|| {
                self.ui.add(V_STACK).nest(|| {
                    self.ui.add(TEXT.static_text("Drag me around").text_selectable(false));
                    self.ui.add(BUTTON.static_text("Button on the glass"));
                });
            });

            if let Some(drag) = self.ui.is_dragged(key) {
                self.glass_pos[i].0 += drag.absolute_delta.x;
                self.glass_pos[i].1 += drag.absolute_delta.y;
            }
        }
    }

    fn render(&mut self, encoder: &mut CommandEncoder, surface_texture: &Texture, surface_view: &TextureView) {
        let render_commands = self.ui.render_commands().to_vec();
        let scale_factor = self.ui.scale_factor();
        let screen_size = [self.config.width as f32, self.config.height as f32];

        self.ui.begin_custom_render();

        begin_pass(encoder, surface_view, LoadOp::Clear(wgpu::Color::BLACK));

        for command in render_commands {
            match command {
                RenderCommand::Keru(range) => {
                    let mut pass = begin_pass(encoder, surface_view, LoadOp::Load);
                    self.ui.render_range(&mut pass, range);
                }
                RenderCommand::CustomRenderingArea { key: _, rect } => {
                    // Copy what was drawn so far, so that the glass shader can sample it. This copies the whole texture for simplicity: copying only the rect plus the blur margin would be faster.
                    encoder.copy_texture_to_texture(
                        surface_texture.as_image_copy(),
                        self.backdrop.as_image_copy(),
                        Extent3d { width: self.config.width, height: self.config.height, depth_or_array_layers: 1 },
                    );

                    let immediates: [f32; 8] = [
                        rect.x[0], rect.y[0],
                        rect.x[1], rect.y[1],
                        screen_size[0], screen_size[1],
                        GLASS_CORNER_RADIUS * scale_factor,
                        scale_factor,
                    ];

                    let mut pass = begin_pass(encoder, surface_view, LoadOp::Load);
                    pass.set_pipeline(&self.glass_pipeline);
                    pass.set_bind_group(0, &self.backdrop_bind_group, &[]);
                    pass.set_immediates(0, bytemuck::cast_slice(&immediates));
                    pass.draw(0..4, 0..1);
                }
            }
        }

        self.ui.finish_custom_render();
    }
}

fn begin_pass<'a>(encoder: &'a mut CommandEncoder, view: &'a TextureView, load: LoadOp<wgpu::Color>) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Liquid glass example pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations { load, store: StoreOp::Store },
            depth_slice: None,
        })],
        ..Default::default()
    })
}

// The backdrop texture holds a copy of the surface, and the glass shader samples it.
fn create_backdrop(device: &Device, config: &SurfaceConfiguration, layout: &BindGroupLayout, sampler: &Sampler) -> (Texture, BindGroup) {
    let backdrop = device.create_texture(&TextureDescriptor {
        label: Some("Backdrop texture"),
        size: Extent3d { width: config.width.max(1), height: config.height.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: config.format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let backdrop_view = backdrop.create_view(&TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Liquid glass bind group"),
        layout,
        entries: &[
            BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&backdrop_view) },
            BindGroupEntry { binding: 1, resource: BindingResource::Sampler(sampler) },
        ],
    });

    (backdrop, bind_group)
}

// Regular boilerplate for the winit loop and wgpu rendering.
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let state = State::new(window, instance);
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        state.ui.window_event(&event, &state.window);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {

                if state.ui.should_update() {
                    state.ui.begin_frame();
                    state.update_ui();
                    state.ui.finish_frame();
                }

                let output = match state.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                state.render(&mut encoder, &output.texture, &view);

                state.ui.submit_commands(encoder.finish());
                state.ui.present(output);
            }
            _ => {}
        }

        if state.ui.should_request_redraw() {
            state.window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application { state: None };
    let _ = event_loop.run_app(&mut app);
}
