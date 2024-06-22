use std::cmp::max;

use bytemuck::{Pod, Zeroable};
use wgpu::{BindGroup, ColorTargetState, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, RenderPass, RenderPipeline, Texture, TextureAspect};
use winit::{dpi::PhysicalPosition, event::{ElementState, Event, MouseButton, WindowEvent}};

use crate::{ui::Xy, BASE_HEIGHT, BASE_WIDTH, SWAPCHAIN_FORMAT};
use crate::ui::Axis::{X, Y};

#[derive(Clone, Copy, Debug)]
#[derive(Zeroable, Pod)]
#[repr(C)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl PixelColor {
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        return Self { r, g, b, a }
    }

    pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        return Self::rgba_u8((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8,);
    }

    pub fn to_f32s(self) -> PixelColorF32 {
        return PixelColorF32 {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: self.a as f32 / 255.0,
        }
    }
}

pub struct PixelColorF32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl PixelColorF32 {
    pub fn to_u8s(self) -> PixelColor {
        return PixelColor {
            r: (self.r * 255.0) as u8,
            g: (self.g * 255.0) as u8,
            b: (self.b * 255.0) as u8,
            a: (self.a * 255.0) as u8,
        }
    }

    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        return PixelColorF32 {
            r,
            g,
            b,
            a,
        }
    }
}

#[derive(Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<PixelColor>,

    mouse_dots: Vec<PhysicalPosition<f64>>,

    // todo: doesn't UI also keep this? maybe its good to keep them separately doe
    last_position: PhysicalPosition<f64>,

    needs_sync: bool,
    needs_render: bool,

    texture: Texture,
    render_pipeline: RenderPipeline,
    texture_bind_group: BindGroup,

    is_drawing: bool,
}

impl Canvas {
    // Create a new canvas with the given width and height, initialized to a background color
    pub fn new(width: usize, height: usize, device: &Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });
        
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
    
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: SWAPCHAIN_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        let mut canvas = Canvas {
            width,
            height,
            pixels: vec![PixelColor::rgba_f32(1.0, 1.0, 1.0, 1.0); width * height],
            texture,
            render_pipeline,
            texture_bind_group,

            last_position: PhysicalPosition::default(),

            mouse_dots: Vec::new(),

            needs_sync: true,
            needs_render: true,
            is_drawing: false,
        };

        for x in 0..width {
            for y in 0..height {
                *canvas.pixel(x, y) = PixelColor::rgba_f32(x as f32 / width as f32, 0.0, y as f32 / height as f32, 1.0);
            }
        }

        return canvas;
    }

    // Set a pixel to a specific color
    pub fn paint_pixel(&mut self, x: usize, y: usize, paint_color: PixelColorF32, brush_alpha: f32) {

        
        // if x < self.width && y < self.height {
        // }
        let index = y * self.width + x;
        let old_color = self.pixels[index].to_f32s();

        if brush_alpha > 0.0 {

            
            let new_color = PixelColorF32 {
               
                r: old_color.r * (1.0 - brush_alpha) + paint_color.r * (brush_alpha),
                g: old_color.g * (1.0 - brush_alpha) + paint_color.g * (brush_alpha),
                b: old_color.b * (1.0 - brush_alpha) + paint_color.b * (brush_alpha),
                
                a: 1.0,
            };
            
            
            self.pixels[index] = new_color.to_u8s();
        }
    }

    // Get the color of a specific pixel
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<PixelColor> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.pixels[index])
        } else {
            None
        }
    }

    // Fill the canvas with a specific color
    pub fn fill(&mut self, color: PixelColor) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
    }

    pub fn update(&mut self) {
        self.draw_dots();
    }

    pub fn draw_dots(&mut self) {
        if self.mouse_dots.len() == 0 {
            return;
        }

        if self.mouse_dots.len() == 1 {
            let first_dot = self.mouse_dots[0];
            let center = Xy::new(first_dot.x, (self.height as f64) - first_dot.y);
            self.draw_circle(center.x as isize, center.y as isize);

            self.mouse_dots.clear();
            return;
        }

        for i in 0..(self.mouse_dots.len() - 1) {
        // for i in 0..self.mouse_dots.len() {
            let first_dot = self.mouse_dots[i];
            let second_dot = self.mouse_dots[i + 1];

            let first_center = Xy::new(first_dot.x, (self.height as f64) - first_dot.y);
            let second_center = Xy::new(second_dot.x, (self.height as f64) - (second_dot.y));
            
            let mut x0 = first_center.x as isize;
            let mut y0 = first_center.y as isize;
            let x1 = second_center.x as isize;
            let y1 = second_center.y as isize;

            let dx = (x1 - x0).abs();
            let dy = -(y1 - y0).abs();
            let sx = if x0 < x1 { 1 } else { -1 };
            let sy = if y0 < y1 { 1 } else { -1 };
            let mut err = dx + dy;
        
            // loop uses isize only, maybe could be more precise or something
            loop {
                // draw           
                self.draw_circle(x0, y0);
                
                // line alg
                if x0 == x1 && y0 == y1 { break; }
                
                let e2 = 2 * err;
                
                if e2 >= dy {
                    err += dy;
                    x0 += sx;
                }
                
                if e2 <= dx {
                    err += dx;
                    y0 += sy;
                }
            }


        }

        let last_element = self.mouse_dots.pop().unwrap();
        self.mouse_dots.clear();
        self.mouse_dots.push(last_element);
    }

    pub fn draw_circle(&mut self, x0: isize, y0: isize) {
        let radius: f64 = 5.0;
        let pixel_radius = (radius as isize) + 2; // some more pixels for antialiasing? 

        for dx in (-pixel_radius)..pixel_radius {
            for dy in (-pixel_radius)..pixel_radius {
                let pixel_x = max(x0 - dx, 0) as usize;
                let pixel_y = max(y0 - dy, 0) as usize;
                let pixel = Xy::new(pixel_x, pixel_y);
                let center = Xy::new(pixel_x as f64, pixel_y as f64);

                let pos = center + (dx as f64, dy as f64);

                let alpha = radius as f64 - ((center - pos).x.powi(2) + (center - pos).y.powi(2)).sqrt();
                let alpha = alpha.clamp(0.0, 1.0);

                let paint_color = PixelColorF32::new(0.2, 0.8, 0.2, 1.0);
                self.paint_pixel(pixel.x, pixel.y, paint_color, alpha as f32);
            }
        }
    }

    pub fn render<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>, queue: &Queue, ) {
        // if self.needs_sync {
            let data = bytemuck::cast_slice(&self.pixels[..]);
            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width as u32 * 4),
                    rows_per_image: Some(self.height as u32),
                },
                Extent3d {
                    width: BASE_WIDTH as u32,
                    height: BASE_HEIGHT as u32,
                    depth_or_array_layers: 1,
                },
            );
            self.needs_sync = false;
        // }

        // if self.needs_render {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            
            self.needs_render = false;
        // }
    }

    pub fn handle_events(&mut self, full_event: &winit::event::Event<()>) {
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Left {
                        self.is_drawing = *state == ElementState::Pressed;
                        self.mouse_dots.push(self.last_position);
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    self.last_position = *position;

                    if self.is_drawing {
                        self.mouse_dots.push(*position);
                    }
                },
            _ => {}
            }
        }
    }

    pub fn pixel(&mut self, x: usize, y: usize) -> &mut PixelColor {
        return &mut self.pixels[y * self.width + x];
    }

}


