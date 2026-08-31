use std::{borrow::Cow, f32::consts::PI};

use keru_draw::*;
use keru_text::{ColorBrush, TextStyle2, parley::{FontFamily, FontFamilyName}};
use winit::{
    dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, window::Window,
};

struct App {
    window: Option<std::sync::Arc<Window>>,
    state: Option<State>,
}

struct State {
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    renderer: Renderer,
}

impl State {
    async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let mut renderer = Renderer::new(&device, &queue, surface_format);

        let style = renderer.text.add_style(
            TextStyle2 {
                font_size: 14.0,
                brush: ColorBrush([200, 200, 200, 255]),
                font_family: FontFamily::Single(FontFamilyName::Named(Cow::Borrowed("sans-serif"))),
                ..Default::default()
            },
            None,
        );
        let _ = style;

        Self { device, surface, config, size, renderer }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        self.renderer.begin_frame();
        self.renderer.prepare_text();

        let pad = 20.0f32;
        let cell = 120.0f32;
        let col = |c: usize| pad + c as f32 * (cell + pad);
        let row = |r: usize| pad + r as f32 * (cell + pad);
        let cx = |c: usize| col(c) + cell * 0.5;
        let cy = |r: usize| row(r) + cell * 0.5;

        // --- Row 0: Rectangle ---

        self.renderer.draw_box(Rectangle {
            top_left: [col(0), row(0)],
            size: [cell, cell],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::new(0.3, 0.6, 1.0, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_box(Rectangle {
            top_left: [col(1), row(0)],
            size: [cell, cell],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 4.0,
            fill: ColorFill::Color(Color::new(0.3, 0.6, 1.0, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_box(Rectangle {
            top_left: [col(2), row(0)],
            size: [cell, cell],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 24.0,
            fill: ColorFill::Color(Color::new(0.3, 0.6, 1.0, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_box(Rectangle {
            top_left: [col(3), row(0)],
            size: [cell, cell],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 8.0,
            fill: ColorFill::Color(Color::new(0.3, 0.6, 1.0, 1.0)),
            blur: 10.0,
            texture: None,
            texture_options: None,
        });

        let grad = self.renderer.create_gradient(Gradient::linear(
            [col(4), row(0)], [col(4) + cell, row(0) + cell],
            Color::new(1.0, 0.3, 0.5, 1.0), Color::new(0.3, 1.0, 0.8, 1.0),
        ));
        self.renderer.draw_box(Rectangle {
            top_left: [col(4), row(0)],
            size: [cell, cell],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 10.0,
            fill: ColorFill::SharedGradient(grad),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // --- Row 1: Circle / Ring ---

        self.renderer.draw_circle(Circle {
            center: [cx(0), cy(1)],
            radius: cell * 0.45,
            fill: ColorFill::Color(Color::new(1.0, 0.6, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_ring(CircleRing {
            center: [cx(1), cy(1)],
            inner_radius: cell * 0.45 - 4.0,
            outer_radius: cell * 0.45,
            fill: ColorFill::Color(Color::new(1.0, 0.6, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            dash_length: None,
            dash_offset: 0.0,
        });

        self.renderer.draw_ring(CircleRing {
            center: [cx(2), cy(1)],
            inner_radius: cell * 0.45 - 30.0,
            outer_radius: cell * 0.45,
            fill: ColorFill::Color(Color::new(1.0, 0.6, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            dash_length: None,
            dash_offset: 0.0,
        });

        self.renderer.draw_ring(CircleRing {
            center: [cx(3), cy(1)],
            inner_radius: cell * 0.45 - 10.0,
            outer_radius: cell * 0.45,
            fill: ColorFill::Color(Color::new(1.0, 0.6, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            dash_length: Some(14.0),
            dash_offset: 0.0,
        });

        self.renderer.draw_ring(CircleRing {
            center: [cx(4), cy(1)],
            inner_radius: cell * 0.45 - 12.0,
            outer_radius: cell * 0.45,
            fill: ColorFill::Color(Color::new(1.0, 0.6, 0.2, 1.0)),
            blur: 10.0,
            texture: None,
            texture_options: None,
            dash_length: None,
            dash_offset: 0.0,
        });

        let grad = self.renderer.create_gradient(Gradient::radial(
            [cx(5), cy(1)], cell * 0.45, cell * 0.45 - 14.0,
            Color::new(1.0, 0.9, 0.2, 1.0), Color::new(1.0, 0.3, 0.7, 1.0),
        ));
        self.renderer.draw_ring(CircleRing {
            center: [cx(5), cy(1)],
            inner_radius: cell * 0.45 - 14.0,
            outer_radius: cell * 0.45,
            fill: ColorFill::SharedGradient(grad),
            blur: 0.0,
            texture: None,
            texture_options: None,
            dash_length: None,
            dash_offset: 0.0,
        });

        // --- Row 2: Triangle ---

        let tri = |cx: f32, cy: f32, r: f32| -> [[f32; 2]; 3] {
            [
                [cx, cy - r],
                [cx - r * 0.866, cy + r * 0.5],
                [cx + r * 0.866, cy + r * 0.5],
            ]
        };

        let [p0, p1, p2] = tri(cx(0), cy(2), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 0.0,
            corner_radius: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let [p0, p1, p2] = tri(cx(1), cy(2), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 4.0,
            corner_radius: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let [p0, p1, p2] = tri(cx(2), cy(2), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 22.0,
            corner_radius: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let [p0, p1, p2] = tri(cx(3), cy(2), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 8.0,
            corner_radius: 0.0,
            blur: 8.0,
            texture: None,
            texture_options: None,
        });

        let grad = self.renderer.create_gradient(Gradient::linear(
            [cx(4), cy(2) - cell * 0.44], [cx(4), cy(2) + cell * 0.44],
            Color::new(0.2, 1.0, 0.5, 1.0), Color::new(0.2, 0.5, 1.0, 1.0),
        ));
        let [p0, p1, p2] = tri(cx(4), cy(2), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::SharedGradient(grad),
            stroke_thickness: 10.0,
            corner_radius: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // --- Row 3: Rounded triangles and hexagons ---

        let [p0, p1, p2] = tri(cx(0), cy(3), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 0.0,
            corner_radius: 12.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let [p0, p1, p2] = tri(cx(1), cy(3), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 5.0,
            corner_radius: 12.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let [p0, p1, p2] = tri(cx(2), cy(3), cell * 0.44);
        self.renderer.draw_triangle(Triangle {
            p0, p1, p2,
            fill: ColorFill::Color(Color::new(0.5, 1.0, 0.4, 1.0)),
            stroke_thickness: 18.0,
            corner_radius: 12.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_hexagon(Hexagon {
            center: [cx(3), cy(3)],
            size: cell * 0.44,
            rotation: PI / 6.0,
            fill: ColorFill::Color(Color::new(0.8, 0.5, 1.0, 1.0)),
            stroke_thickness: 0.0,
            corner_radius: 10.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_hexagon(Hexagon {
            center: [cx(4), cy(3)],
            size: cell * 0.44,
            rotation: PI / 6.0,
            fill: ColorFill::Color(Color::new(0.8, 0.5, 1.0, 1.0)),
            stroke_thickness: 5.0,
            corner_radius: 10.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_hexagon(Hexagon {
            center: [cx(5), cy(3)],
            size: cell * 0.44,
            rotation: PI / 6.0,
            fill: ColorFill::Color(Color::new(0.8, 0.5, 1.0, 1.0)),
            stroke_thickness: 18.0,
            corner_radius: 10.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // --- Row 4: Capsule, Hexagon, Bezier ---

        let seg_y = cy(4);
        let seg_h = cell * 0.35;
        let seg_t = cell * 0.38;

        self.renderer.draw_segment(Segment {
            start: [cx(0), seg_y - seg_h],
            end: [cx(0), seg_y + seg_h],
            thickness: seg_t * 2.0,
            stroke_thickness: 0.0,
            fill: ColorFill::Color(Color::new(1.0, 0.3, 0.7, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [cx(1), seg_y - seg_h],
            end: [cx(1), seg_y + seg_h],
            thickness: seg_t * 2.0,
            stroke_thickness: 4.0,
            fill: ColorFill::Color(Color::new(1.0, 0.3, 0.7, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [cx(2), seg_y - seg_h],
            end: [cx(2), seg_y + seg_h],
            thickness: seg_t * 2.0,
            stroke_thickness: 22.0,
            fill: ColorFill::Color(Color::new(1.0, 0.3, 0.7, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [cx(3), seg_y - seg_h],
            end: [cx(3), seg_y + seg_h],
            thickness: seg_t * 2.0,
            stroke_thickness: 10.0,
            fill: ColorFill::Color(Color::new(1.0, 0.3, 0.7, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 10.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_hexagon(Hexagon {
            center: [cx(4), cy(4)],
            size: cell * 0.44,
            rotation: PI / 6.0,
            fill: ColorFill::Color(Color::new(0.8, 0.5, 1.0, 1.0)),
            stroke_thickness: 10.0,
            corner_radius: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        let bx = cx(5);
        let by = cy(4);
        let br = cell * 0.38;
        self.renderer.draw_quadratic_bezier(QuadraticBezier {
            p0: [bx - br, by + br * 0.5],
            p1: [bx, by - br],
            p2: [bx + br, by + br * 0.5],
            thickness: 22.0,
            stroke_thickness: 8.0,
            blur: 0.0,
            color: Color::new(0.3, 0.8, 1.0, 1.0),
        });

        // --- Row 5: Circle Arc ---

        self.renderer.draw_arc(CircleArc {
            center: [cx(0), cy(5)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 4.0,
            fill: ColorFill::Color(Color::new(0.2, 0.8, 1.0, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_arc(CircleArc {
            center: [cx(1), cy(5)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 20.0,
            fill: ColorFill::Color(Color::new(0.2, 0.8, 1.0, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_arc(CircleArc {
            center: [cx(2), cy(5)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 20.0,
            fill: ColorFill::Color(Color::new(0.2, 0.8, 1.0, 1.0)),
            dash_length: Some(16.0),
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_arc(CircleArc {
            center: [cx(3), cy(5)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 10.0,
            fill: ColorFill::Color(Color::new(0.2, 0.8, 1.0, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 8.0,
            texture: None,
            texture_options: None,
        });

        let grad = self.renderer.create_gradient(Gradient::linear(
            [cx(4) - cell * 0.45, cy(5)], [cx(4) + cell * 0.45, cy(5)],
            Color::new(0.2, 0.8, 1.0, 1.0), Color::new(0.8, 0.2, 1.0, 1.0),
        ));
        self.renderer.draw_arc(CircleArc {
            center: [cx(4), cy(5)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 14.0,
            fill: ColorFill::SharedGradient(grad),
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // --- Row 6: Pie ---

        // filled
        self.renderer.draw_pie(CirclePie {
            center: [cx(0), cy(6)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 0.75,
            fill: ColorFill::Color(Color::new(1.0, 0.5, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        // stroke only
        self.renderer.draw_pie(CirclePie {
            center: [cx(1), cy(6)],
            radius: cell * 0.45,
            start_angle: -PI * 0.5,
            end_angle: PI * 0.5,
            fill: ColorFill::Color(Color::new(1.0, 0.5, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 5.0,
            corner_radius: 0.0,
        });

        // rounded corners
        self.renderer.draw_pie(CirclePie {
            center: [cx(2), cy(6)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 0.75,
            fill: ColorFill::Color(Color::new(1.0, 0.5, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 14.0,
        });

        // rounded + stroke
        self.renderer.draw_pie(CirclePie {
            center: [cx(3), cy(6)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 0.75,
            fill: ColorFill::Color(Color::new(1.0, 0.5, 0.2, 1.0)),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 5.0,
            corner_radius: 10.0,
        });

        let grad = self.renderer.create_gradient(Gradient::radial(
            [cx(4), cy(6)], cell * 0.45, 0.0,
            Color::new(1.0, 0.9, 0.2, 1.0), Color::new(1.0, 0.2, 0.5, 1.0),
        ));
        // gradient + blur
        self.renderer.draw_pie(CirclePie {
            center: [cx(4), cy(6)],
            radius: cell * 0.45,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            fill: ColorFill::SharedGradient(grad),
            blur: 8.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        self.renderer.draw_text_decorations();

        self.renderer.autorender(&self.surface, wgpu::Color { r: 0.12, g: 0.12, b: 0.14, a: 1.0 });
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_inner_size(PhysicalSize::new(1600, 900))
                            .with_title("strokes"),
                    )
                    .unwrap(),
            );
            window.set_ime_allowed(true);
            let state = pollster::block_on(State::new(window.clone()));
            self.window = Some(window);
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(state)) = (&self.window, &mut self.state) {
            state.renderer.text.handle_event(&event, window);

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    state.render();
                    window.request_redraw();
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        window: None,
        state: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
