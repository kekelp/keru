// When using Keru, you remain in control of your program's wgpu rendering, so it's very easy to draw custom rendered content below or above the Keru Ui.
//
// But what if we wanted to draw custom rendered stuff *between* Ui elements, with proper z-ordering and transparency?
// 
// This example shows the system that allows it.
//
// - when declaring the Ui, we can mark specific nodes with `.custom_render(true)`
// - when preparing its render data, the Ui pays attention to which nodes need custom rendering, 
//    and determines the properly ordered sequence of Ui element ranges and custom render content.
//    Example: [ ui_elements_background ] [ custom_1 ] [ ui_elements_middle ] [ custom_2 ] [ ui_elements_foreground ]
// - we get the sequence with Ui::render_commands().
// - we go through the sequence and perform the render commands depending on what we find:
//     - ui_elements_xxx is a range of Ui elements. We just pass them to the function ui.render_range(ui_elements_xxx)
//     - custom_xxx contains the screen rect of the special node. We draw whatever we want in the rect.


use keru::*;
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
    panel_pos: (f64, f64),
    custom_pipeline: RenderPipeline,
}

impl State {
    fn new(window: Arc<Window>, instance: Instance) -> Self {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        // Note that push constants are not guaranteed to be supported everywhere.
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_features: Features::PUSH_CONSTANTS,
            required_limits: Limits { max_push_constant_size: 32, ..Default::default() },
            ..Default::default()
        })).unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| ! f.is_srgb())
            .copied().unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &config);

        let ui = Ui::new(&device, &queue, &config);

        // Wgpu boilerplate to set up a custom shader and a pipeline for it
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Custom Shader"),
            source: ShaderSource::Wgsl(include_str!("custom_rendering_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Custom Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX_FRAGMENT,
                range: 0..32,
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Custom Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
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
            multiview: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            config,
            ui,
            panel_pos: (50.0, 50.0),
            custom_pipeline: pipeline,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    #[node_key] const BACK_PANEL: NodeKey;
    #[node_key] const HEADER: NodeKey;
    #[node_key] const CUSTOM_RECT: NodeKey;
    #[node_key] const OVERLAY_LABEL: NodeKey;

    fn update_ui(&mut self) {

        let panel = PANEL
            .padding(30)
            .position_x(Position::Static(Len::Pixels(self.panel_pos.0 as u32)))
            .position_y(Position::Static(Len::Pixels(self.panel_pos.1 as u32)))
            .sense_drag(true)
            .key(Self::BACK_PANEL);

        let custom_rect = DEFAULT
            .invisible()
            .custom_render(true)
            .size_symm(Size::Pixels(300))
            .key(Self::CUSTOM_RECT);

        let button = BUTTON
            .position_x(Position::Static(Len::Frac(0.6)))
            .text("Overlay button\ndrawn over it");

        self.ui.add(panel).nest(|| {
            self.ui.add(V_STACK).nest(|| {
                self.ui.static_paragraph("Background panel,\nrendered below the custom shader rect");
                
                self.ui.add(custom_rect).nest(|| {
                    self.ui.add(button);
                });
                
                self.ui.static_paragraph("Click and drag the panel to move it.");
            })
        });

        if let Some(drag) = self.ui.is_dragged(Self::BACK_PANEL) {
            self.panel_pos.0 -= drag.absolute_delta.x;
            self.panel_pos.1 -= drag.absolute_delta.y;
            self.panel_pos.0 = f64::clamp(self.panel_pos.0, 0.0, 100000.0);
            self.panel_pos.1 = f64::clamp(self.panel_pos.1, 0.0, 100000.0);
        }
    }

    fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // Get a list of rendering commands that we need to do to render the ui elements and our custom ones in the proper order.
        let render_commands = self.ui.render_commands().to_vec();

        self.ui.begin_custom_render();

        for command in render_commands {
            match command {
                RenderCommand::Keru(range) => {
                    // Render the regular UI elements for this range.
                    self.ui.render_range(render_pass, range);
                }
                RenderCommand::CustomRenderingArea { key: _, rect } => {
                    // Do our custom rendering. If there were multiple custom rendered rects, we could tell them apart by key.
                    let push_constants: [f32; 8] = [
                        rect.x[0], rect.y[0],
                        rect.x[1], rect.y[1],
                        self.ui.ui_time(),
                        0.0, 0.0, 0.0,
                    ];
                    render_pass.set_pipeline(&self.custom_pipeline);
                    render_pass.set_push_constants(ShaderStages::VERTEX_FRAGMENT, 0, bytemuck::cast_slice(&push_constants));
                    render_pass.draw(0..4, 0..1);
                }
            }
        }

        self.ui.finish_custom_render();
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        window.set_ime_allowed(true);
        let instance = Instance::new(&InstanceDescriptor::default());
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

                let output = state.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("custom render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations::default(),
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });

                    state.render(&mut render_pass);
                }

                state.ui.submit_commands(encoder.finish());
                output.present();
            }
            _ => {}
        }

        state.window.request_redraw();
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application { state: None };
    let _ = event_loop.run_app(&mut app);
}
