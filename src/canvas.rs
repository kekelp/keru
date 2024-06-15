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

    fn blend(old_color: PixelColor, new_color: PixelColor) -> PixelColor {
        let old_color = old_color.to_f32s();
        let new_color = new_color.to_f32s();

        let new_a = new_color.a + old_color.a * (new_color.a - 1.0);
        return PixelColorF32 {
            r: old_color.r * (1.0 - new_a) + new_color.r * new_a,
            g: old_color.g * (1.0 - new_a) + new_color.g * new_a,
            b: old_color.b * (1.0 - new_a) + new_color.b * new_a,
            a: new_a,
        }.to_u8s()
    }

    pub fn to_f32s(self) -> PixelColorF32 {
        return PixelColorF32 {
            r: self.r as f32 / 255.0,
            g: self.r as f32 / 255.0,
            b: self.r as f32 / 255.0,
            a: self.r as f32 / 255.0,
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
            g: (self.r * 255.0) as u8,
            b: (self.r * 255.0) as u8,
            a: (self.r * 255.0) as u8,
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
        
        Canvas {
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
        }
    }

    // Set a pixel to a specific color
    pub fn paint_pixel(&mut self, x: usize, y: usize, color: PixelColor) {
        // if x < self.width && y < self.height {
        // }
        let index = y * self.width + x;
        let old_color = self.pixels[index];

        let new_color = PixelColor::blend(old_color, color);
        self.pixels[index] = color;
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
        // todo: might be stupid
        // if self.mouse_dots.len() == 1 {
        //     self.mouse_dots.push(self.mouse_dots[0])
        // }
        for i in 0..self.mouse_dots.len() {
            let first_dot = self.mouse_dots[i];
            // let second_dot = self.mouse_dots[i];

            let center = Xy::new(first_dot.x, (self.height as f64) - first_dot.y);
            let center_pixel = Xy::new(first_dot.x as usize, self.height - (first_dot.y as usize));
            // let second_dot = Xy::new(second_dot.x as usize, self.height - (second_dot.y as usize));

            let radius: f64 = 80.0;
            let radius_squared = radius.powi(2);
            let pixel_radius = (radius as isize) + 2; // some more pixels for antialiasing? 

            for dx in (-pixel_radius)..pixel_radius {
                for dy in (-pixel_radius)..pixel_radius {
                    let pixel_x = max(center_pixel.x as isize - dx, 0) as usize;
                    let pixel_y = max(center_pixel.y as isize - dy, 0) as usize;
                    let pixel = Xy::new(pixel_x, pixel_y);

                    let pos = center + (dx as f64, dy as f64);

                    let alpha = radius as f64 - ((center - pos).x.powi(2) + (center - pos).y.powi(2)).sqrt();
                    let alpha = (alpha * 255.) as u8;
                    self.paint_pixel(pixel.x, pixel.y, PixelColor::rgba_u8(alpha, 0, 0, 1));
                    // let alpha = (alpha).clamp(0.0, 1.0);
                    // println!("  {:?}", alpha);
                    // let alpha = (alpha * 255.0) as u8;
                    // if alpha > 0 {
                    // }
                }
            }
        }

        self.mouse_dots.clear();
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


}
