use std::{borrow::Cow, f32::consts::PI};

use keru_draw::*;
use keru_text::{ColorBrush, TextStyle2, Transform2D, parley::{FontFamily, FontFamilyName}};
use winit::{
    dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, window::Window,
};

const TIGER_SVG: &[u8] = include_bytes!("tiger.svg");
const TEXTURE: &[u8] = include_bytes!("texture.jpg");
const NINE_SLICE_IMAGE: &[u8] = include_bytes!("nine-slice-test.png");

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
    text_edit: TextEditHandle,
    text_box1: TextBoxHandle,
    text_box2: TextBoxHandle,
    text_box3: TextBoxHandle,
    svg_handle: LoadedImage,
    texture_handle: LoadedImage,
    nine_slice_handle: LoadedImage,
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
            .find(|f| ! f.is_srgb())
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
                font_size: 18.0,
                brush: ColorBrush([0, 0, 0, 255]),
                // font_family: FontFamily::Single(Cow::Borrowed("sans-serif")),
                font_family: FontFamily::Single(FontFamilyName::Named(Cow::Borrowed("sans-serif"))),
                ..Default::default()
            },
            None,
        );
        let text_edit = renderer.text.add_text_edit(
            "🌈Bottom text".to_owned(),
            Some((500.0, 400.0)),
            (280.0, 150.0),
            0.0,
        );
        renderer.text.get_text_edit_mut(&text_edit).set_style(&style);

        let text_box1 = renderer.text.add_text_box(
            "Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.".to_owned(),
            Some((0.0, 0.0)),
            (200.0, 50.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box1).set_style(&style);
        renderer.text.get_text_box_mut(&text_box1).set_transform(Transform2D {
            translation: (40.0, 630.0),
            rotation: PI * -0.1,
            scale: 1.2,
        });

        let text_box2 = renderer.text.add_text_box(
            "90 degree rotations are fine.".to_owned(),
            Some((0.0, 0.0)),
            (200.0, 60.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box2).set_style(&style);
        renderer.text.get_text_box_mut(&text_box2).set_transform(Transform2D {
            translation: (400.0, 580.0),
            rotation: PI * 0.5,
            scale: 1.0,
        });

        let text_box3 = renderer.text.add_text_box(
            "What kind of renderer doesn't have a hex grid primitive?".to_owned(),
            Some((750.0, 600.0)),
            (250.0, 100.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box3).set_style(&style);


        let svg_handle = renderer.image_renderer.load_svg(TIGER_SVG, 200, 200).unwrap();
        let texture_handle = renderer.image_renderer.load_encoded_image(TEXTURE).unwrap();
        let nine_slice_handle = renderer.image_renderer.load_encoded_image(NINE_SLICE_IMAGE).unwrap();

        Self { device, surface, config, size, renderer, text_edit, text_box1, text_box2, text_box3, svg_handle, texture_handle, nine_slice_handle }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // self.renderer.resize(new_size.width, new_size.height);
        }
    }

    fn render(&mut self) {
        self.renderer.begin_frame();
        self.renderer.prepare_text();

        let grad_box_horiz = self.renderer.create_gradient(Gradient::linear(
            [20.0, 50.0], [100.0, 50.0],
            Color::new(1.0, 0.3, 0.3, 1.0), Color::new(0.3, 0.3, 1.0, 1.0),
        ));
        let grad_box_diag = self.renderer.create_gradient(Gradient::linear(
            [125.0, 15.0], [195.0, 85.0],
            Color::new(1.0, 0.5, 0.3, 1.0), Color::new(0.3, 1.0, 0.5, 1.0),
        ));
        let grad_circle_radial = self.renderer.create_gradient(Gradient::radial(
            [50.0, 150.0], 20.0, 0.0,
            Color::new(1.0, 1.0, 0.3, 1.0), Color::new(1.0, 0.3, 0.3, 1.0),
        ));
        let grad_circle_linear = self.renderer.create_gradient(Gradient::linear(
            [140.0, 120.0], [140.0, 180.0],
            Color::new(0.3, 0.7, 1.0, 1.0), Color::new(1.0, 0.3, 0.7, 1.0),
        ));
        let grad_segment1 = self.renderer.create_gradient(Gradient::linear(
            [370.0, 500.0], [430.0, 540.0],
            Color::new(1.0, 0.9, 0.2, 0.7), Color::new(0.2, 0.9, 1.0, 0.7),
        ));
        let grad_segment2 = self.renderer.create_gradient(Gradient::linear(
            [370.0, 540.0], [430.0, 500.0],
            Color::new(1.0, 0.2, 0.9, 0.8), Color::new(0.2, 1.0, 0.9, 0.8),
        ));
        let grad_grid = self.renderer.create_gradient(Gradient::linear(
            [850.0, 120.0], [1050.0, 120.0],
            Color::new(0.2, 0.2, 1.0, 1.0), Color::new(1.0, 0.2, 0.2, 1.0),
        ));
        let grad_hexagon = self.renderer.create_gradient(Gradient::linear(
            [591.7, 471.7], [648.3, 528.3],
            Color::new(1.0, 0.3, 0.5, 1.0), Color::new(0.5, 0.3, 1.0, 1.0),
        ));

        // Gradient box - horizontal
        self.renderer.draw_box(Rectangle {
            top_left: [20.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::SharedGradient(grad_box_horiz),
            blur: 10.0,
            texture: Some(self.texture_handle),
            texture_options: None,
        });

        // Gradient box - diagonal
        self.renderer.draw_box(Rectangle {
            top_left: [120.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 5.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::SharedGradient(grad_box_diag),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Box with only top corners rounded
        self.renderer.draw_box(Rectangle {
            top_left: [220.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 16.0,
            rounded_corners: RoundedCorners::TOP,
            border_thickness: 5.0,
            fill: ColorFill::Color(Color { r: 0.015686, g: 0.666667, b: 0.427451, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Box with all corners rounded
        self.renderer.draw_box(Rectangle {
            top_left: [320.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 30.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 5.0,
            fill: ColorFill::Color(Color { r: 0.8, g: 1.0, b: 0.3, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Box with only bottom-right corner rounded
        self.renderer.draw_box(Rectangle {
            top_left: [420.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 20.0,
            rounded_corners: RoundedCorners::BOTTOM_RIGHT,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color { r: 0.6, g: 0.3, b: 0.9, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Box with diagonal corners rounded (top-left and bottom-right)
        self.renderer.draw_box(Rectangle {
            top_left: [520.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 15.0,
            rounded_corners: RoundedCorners::TOP_LEFT | RoundedCorners::BOTTOM_RIGHT,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color { r: 0.9, g: 0.5, b: 0.2, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Blurred boxes
        self.renderer.draw_box(Rectangle {
            top_left: [650.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 8.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::new(0.2, 0.6, 1.0, 1.0)),
            blur: 5.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_box(Rectangle {
            top_left: [680.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 8.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::new(1.0, 0.3, 0.5, 1.0)),
            blur: 15.0,
            texture: None,
            texture_options: None,
        });

        // Radial gradient circle
        self.renderer.draw_circle(Circle {
            center: [50.0, 150.0],
            radius: 20.0,
            fill: ColorFill::SharedGradient(grad_circle_radial),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Linear gradient circle
        self.renderer.draw_circle(Circle {
            center: [140.0, 150.0],
            radius: 30.0,
            fill: ColorFill::SharedGradient(grad_circle_linear),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_circle(Circle {
            center: [250.0, 150.0],
            radius: 40.0,
            fill: ColorFill::Color(Color { r: 0.3, g: 0.9, b: 1.0, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Blurred circles
        self.renderer.draw_circle(Circle {
            center: [360.0, 95.0],
            radius: 30.0,
            fill: ColorFill::Color(Color::new(1.0, 0.8, 0.2, 1.0)),
            blur: 8.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_circle(Circle {
            center: [460.0, 95.0],
            radius: 30.0,
            fill: ColorFill::Color(Color::new(0.4, 1.0, 0.5, 1.0)),
            blur: 20.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_ring(CircleRing {
            center: [360.0, 150.0],
            inner_radius: 35.0,
            outer_radius: 40.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 1.0, b: 0.3, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 0.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_ring(CircleRing {
            center: [460.0, 150.0],
            inner_radius: 30.0,
            outer_radius: 45.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.8, b: 0.3, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 0.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_ring(CircleRing {
            center: [560.0, 150.0],
            inner_radius: 25.0,
            outer_radius: 50.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.6, b: 0.3, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: Some(15.0),  // dashed ring example
            blur: 0.0,
            dash_offset: 0.0,
        });

        // Blurred ring
        self.renderer.draw_ring(CircleRing {
            center: [660.0, 150.0],
            inner_radius: 30.0,
            outer_radius: 45.0,
            fill: ColorFill::Color(Color::new(0.3, 0.9, 1.0, 1.0)),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 8.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_arc(CircleArc {
            center: [60.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: PI * 0.5,
            thickness: 8.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.3, b: 1.0, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 0.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_arc(CircleArc {
            center: [170.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: PI,
            thickness: 8.0,
            fill: ColorFill::Color(Color { r: 0.8, g: 0.3, b: 1.0, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 0.0,
            dash_offset: 0.0,
        });

        // dashed arc
        self.renderer.draw_arc(CircleArc {
            center: [280.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 8.0,
            fill: ColorFill::Color(Color { r: 0.6, g: 0.3, b: 1.0, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: Some(10.0),
            blur: 3.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_arc(CircleArc {
            center: [390.0, 280.0],
            radius: 40.0,
            start_angle: PI * 0.25,
            end_angle: PI * 1.25,
            thickness: 8.0,
            fill: ColorFill::Color(Color { r: 0.4, g: 0.3, b: 1.0, a: 1.0 }),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 0.0,
            dash_offset: 0.0,
        });

        // Blurred arc
        self.renderer.draw_arc(CircleArc {
            center: [500.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: PI * 1.5,
            thickness: 10.0,
            fill: ColorFill::Color(Color::new(0.2, 0.4, 1.0, 1.0)),
            texture: None,
            texture_options: None,
            dash_length: None,
            blur: 8.0,
            dash_offset: 0.0,
        });

        self.renderer.draw_pie(CirclePie {
            center: [60.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: PI * 0.25,
            fill: ColorFill::Color(Color { r: 0.3, g: 1.0, b: 1.0, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        self.renderer.draw_pie(CirclePie {
            center: [170.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: PI * 0.5,
            fill: ColorFill::Color(Color { r: 0.3, g: 1.0, b: 0.8, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        self.renderer.draw_pie(CirclePie {
            center: [280.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: PI,
            fill: ColorFill::Color(Color { r: 0.3, g: 1.0, b: 0.6, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        self.renderer.draw_pie(CirclePie {
            center: [390.0, 400.0],
            radius: 45.0,
            start_angle: PI * 0.5,
            end_angle: PI * 2.0,
            fill: ColorFill::Color(Color { r: 0.3, g: 1.0, b: 0.4, a: 1.0 }),
            blur: 0.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        // Blurred pie
        self.renderer.draw_pie(CirclePie {
            center: [490.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: PI * 0.75,
            fill: ColorFill::Color(Color::new(0.9, 0.5, 0.3, 1.0)),
            blur: 10.0,
            texture: None,
            texture_options: None,
            stroke_thickness: 0.0,
            corner_radius: 0.0,
        });

        self.renderer.draw_segment(Segment {
            start: [20.0, 520.0],
            end: [100.0, 520.0],
            thickness: 3.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.5, b: 0.0, a: 1.0 }),
            stroke_thickness: 0.0,
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [120.0, 500.0],
            end: [200.0, 540.0],
            thickness: 6.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.6, b: 0.0, a: 1.0 }),
            stroke_thickness: 0.0,
            dash_length: Some(10.0),
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [230.0, 500.0],
            end: [230.0, 540.0],
            thickness: 10.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.7, b: 0.0, a: 1.0 }),
            stroke_thickness: 0.0,
            dash_length: Some(15.0),
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        self.renderer.draw_segment(Segment {
            start: [260.0, 540.0],
            end: [340.0, 500.0],
            thickness: 8.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 0.8, b: 0.0, a: 1.0 }),
            stroke_thickness: 0.0,
            dash_length: Some(5.0),
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Gradient segments forming an X
        self.renderer.draw_segment(Segment {
            start: [370.0, 500.0],
            end: [430.0, 540.0],
            thickness: 5.0,
            fill: ColorFill::SharedGradient(grad_segment1),
            stroke_thickness: 0.0,
            dash_length: Some(8.0),
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });
        self.renderer.draw_segment(Segment {
            start: [370.0, 540.0],
            end: [430.0, 500.0],
            thickness: 5.0,
            fill: ColorFill::SharedGradient(grad_segment2),
            stroke_thickness: 0.0,
            dash_length: None,
            dash_offset: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Blurred segment
        self.renderer.draw_segment(Segment {
            start: [460.0, 505.0],
            end: [545.0, 535.0],
            thickness: 8.0,
            stroke_thickness: 0.0,
            fill: ColorFill::Color(Color::new(0.3, 0.7, 1.0, 1.0)),
            dash_length: None,
            dash_offset: 0.0,
            blur: 6.0,
            texture: None,
            texture_options: None,
        });

        // Blurred triangle
        self.renderer.draw_triangle(Triangle {
            p0: [480.0, 545.0],
            p1: [440.0, 615.0],
            p2: [520.0, 615.0],
            fill: ColorFill::Color(Color::new(1.0, 0.4, 0.8, 1.0)),
            stroke_thickness: 0.0,
            blur: 8.0,
            texture: Some(self.texture_handle),
            texture_options: None,
            corner_radius: 0.0,
        });

        // Square grid
        self.renderer.draw_grid(Grid {
            top_left: [850.0, 20.0],
            size: [200.0, 200.0],
            lattice_size: [30.0, 20.0],
            offset: [0.0, 0.0],
            line_thickness: 2.0,
            fill: ColorFill::SharedGradient(grad_grid),
            grid_type: GridType::Square,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Hexagonal grid
        self.renderer.draw_grid(Grid {
            top_left: [850.0, 250.0],
            size: [300.0, 300.0],
            lattice_size: [50.0, 50.0],
            offset: [0.0, 0.0],
            line_thickness: 2.0,
            fill: ColorFill::Color(Color::new(1.0, 0.0, 0.0, 0.5)),
            grid_type: GridType::Hexagonal,
            blur: 0.0,
            texture: None,
            texture_options: None,
        });

        // Blurred grid
        self.renderer.draw_grid(Grid {
            top_left: [850.0, 580.0],
            size: [200.0, 150.0],
            lattice_size: [20.0, 20.0],
            offset: [0.0, 0.0],
            line_thickness: 2.0,
            fill: ColorFill::Color(Color::new(0.3, 0.8, 0.5, 1.0)),
            grid_type: GridType::Square,
            blur: 4.0,
            texture: None,
            texture_options: None,
        });

        // Hexagons - solid filled
        self.renderer.draw_hexagon(Hexagon {
            center: [520.0, 400.0 + 100.0],
            size: 40.0,
            rotation: 0.0,
            fill: ColorFill::Color(Color { r: 0.2, g: 0.7, b: 0.9, a: 1.0 }),
            stroke_thickness: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
            corner_radius: 0.0,
        });

        // Hexagon - gradient filled
        self.renderer.draw_hexagon(Hexagon {
            center: [620.0, 400.0 + 100.0],
            size: 40.0,
            rotation: PI / 2.0,
            fill: ColorFill::SharedGradient(grad_hexagon),
            stroke_thickness: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
            corner_radius: 0.0,
        });

        // Hexagon - stroke only
        self.renderer.draw_hexagon(Hexagon {
            center: [520.0, 500.0 + 100.0],
            size: 40.0,
            rotation: 0.0,
            fill: ColorFill::Color(Color { r: 0.9, g: 0.5, b: 0.2, a: 1.0 }),
            stroke_thickness: 4.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
            corner_radius: 0.0,
        });

        // Hexagon - rotated (pointy-top)
        self.renderer.draw_hexagon(Hexagon {
            center: [620.0, 500.0 + 100.0],
            size: 40.0,
            rotation: PI / 6.0, // 30 degrees for pointy-top
            fill: ColorFill::Color(Color { r: 0.5, g: 0.9, b: 0.3, a: 1.0 }),
            stroke_thickness: 0.0,
            blur: 0.0,
            texture: None,
            texture_options: None,
            corner_radius: 0.0,
        });

        // Hexagon with texture
        self.renderer.draw_hexagon(Hexagon {
            center: [570.0, 600.0 + 100.0],
            size: 50.0,
            rotation: 0.0,
            fill: ColorFill::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            stroke_thickness: 0.0,
            blur: 10.0,
            texture: Some(self.texture_handle),
            texture_options: None,
            corner_radius: 0.0,
        });

        // Blurred hexagon
        self.renderer.draw_hexagon(Hexagon {
            center: [720.0, 500.0],
            size: 40.0,
            rotation: 0.0,
            fill: ColorFill::Color(Color::new(0.8, 0.4, 0.2, 1.0)),
            stroke_thickness: 0.0,
            blur: 10.0,
            texture: None,
            texture_options: None,
            corner_radius: 0.0,
        });

        // Dashed box outline (no rounded corners)
        self.renderer.draw_dashed_box_outline(DashedBoxOutline {
            top_left: [735.0, 100.0],
            size: [80.0, 60.0],
            corner_radius: 0.0,
            thickness: 3.0,
            color: Color { r: 1.0, g: 0.5, b: 0.2, a: 1.0 },
            blur: 3.0,
            dash_length: 10.0,
        });

        // Dashed box outline (with rounded corners)
        self.renderer.draw_dashed_box_outline(DashedBoxOutline {
            top_left: [735.0, 180.0],
            size: [80.0, 60.0],
            corner_radius: 15.0,
            thickness: 3.0,
            color: Color { r: 0.2, g: 0.8, b: 1.0, a: 1.0 },
            blur: 0.0,
            dash_length: 8.0,
        });

        // Dashed hexagon outline
        self.renderer.draw_dashed_hexagon_outline(DashedHexagonOutline {
            center: [770.0, 320.0],
            size: 40.0,
            rotation: 0.0,
            thickness: 3.0,
            color: Color { r: 0.8, g: 0.3, b: 1.0, a: 1.0 },
            blur: 3.0,
            dash_length: 12.0,
        });

        // Dashed hexagon outline (rotated)
        self.renderer.draw_dashed_hexagon_outline(DashedHexagonOutline {
            center: [770.0, 420.0],
            size: 40.0,
            rotation: PI / 6.0,
            thickness: 3.0,
            color: Color { r: 0.3, g: 1.0, b: 0.5, a: 1.0 },
            blur: 0.0,
            dash_length: 8.0,
        });

        // Quadratic bezier curve
        self.renderer.draw_quadratic_bezier(QuadraticBezier {
            p0: [720.0, 520.0],
            p1: [770.0, 470.0],
            p2: [820.0, 520.0],
            thickness: 4.0,
            stroke_thickness: 0.0,
            blur: 0.0,
            color: Color { r: 0.9, g: 0.2, b: 0.6, a: 1.0 },
        });

        // Blurred quadratic bezier
        self.renderer.draw_quadratic_bezier(QuadraticBezier {
            p0: [720.0, 570.0],
            p1: [770.0, 520.0],
            p2: [820.0, 570.0],
            thickness: 6.0,
            stroke_thickness: 0.0,
            blur: 8.0,
            color: Color::new(0.2, 0.8, 0.4, 1.0),
        });

        self.renderer.draw_text_box(&self.text_box3);

        self.renderer.draw_text_edit(&self.text_edit);

        // ---- Nine-slice / tiling section (right column, x=1100) ----
        let ns = self.nine_slice_handle;
        let insets = NineSliceMargins::uniform(40.0);
        let x = 1100.0;

        // Box: nine-slice, all-stretch (default)
        self.renderer.draw_box(Rectangle {
            top_left: [x, 20.0],
            size: [450.0, 130.0],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(ns),
            texture_options: Some(TextureOptions {
                nine_slice: Some(insets),
                ..Default::default()
            }),
            blur: 0.0,
        });

        // Box: nine-slice, tile middle horizontally
        self.renderer.draw_box(Rectangle {
            top_left: [x, 170.0],
            size: [450.0, 130.0],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(ns),
            texture_options: Some(TextureOptions {
                nine_slice: Some(insets),
                tile_x: TileMode::Tile,
                ..Default::default()
            }),
            blur: 0.0,
        });

        // Box: nine-slice, tile_fit both axes
        self.renderer.draw_box(Rectangle {
            top_left: [x, 320.0],
            size: [450.0, 130.0],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(ns),
            texture_options: Some(TextureOptions {
                nine_slice: Some(insets),
                tile_x: TileMode::TileFit,
                tile_y: TileMode::TileFit,
            }),
            blur: 0.0,
        });

        // Box: no nine-slice, tile both axes (repeating texture)
        self.renderer.draw_box(Rectangle {
            top_left: [x, 470.0],
            size: [200.0, 200.0],
            corner_radius: 12.0,
            rounded_corners: RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(self.texture_handle),
            texture_options: Some(TextureOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            }),
            blur: 0.0,
        });

        // Circle: nine-slice stretch
        self.renderer.draw_circle(Circle {
            center: [x + 310.0, 560.0],
            radius: 90.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(ns),
            texture_options: Some(TextureOptions {
                nine_slice: Some(insets),
                ..Default::default()
            }),
            blur: 0.0,
        });

        // Triangle: nine-slice, tile both
        self.renderer.draw_triangle(Triangle {
            p0: [x + 70.0, 700.0],
            p1: [x, 840.0],
            p2: [x + 140.0, 840.0],
            fill: ColorFill::Color(Color::WHITE),
            stroke_thickness: 0.0,
            texture: Some(ns),
            texture_options: Some(TextureOptions {
                nine_slice: Some(insets),
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            }),
            blur: 0.0,
            corner_radius: 0.0,
        });

        // SVG rendered as image
        self.renderer.draw_image(self.svg_handle, 520.0, 150.0, 180.0, 180.0);

        // Rotated text
        self.renderer.text.get_text_box_mut(&self.text_box1);
        self.renderer.draw_text_box(&self.text_box1);

        self.renderer.draw_text_box(&self.text_box2);
        
        self.renderer.draw_text_decorations();

        self.renderer.autorender(&self.surface, wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
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
                            .with_title("example"),
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
            // Register window with text system
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
