use std::cmp::max;

use bytemuck::{Pod, Zeroable};
use wgpu::{BindGroup, ColorTargetState, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, RenderPass, RenderPipeline, Texture, TextureAspect};
use winit::{dpi::PhysicalPosition, event::{ElementState, Event, MouseButton, WindowEvent}};

use crate::{ui::Xy, BASE_HEIGHT, BASE_WIDTH, SWAPCHAIN_FORMAT};
use crate::ui::Axis::{X, Y};

#[derive(Clone, Copy, Debug)]
#[derive(Zeroable, Pod)]
#[repr(C)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        return Self { r, g, b, a }
    }

    pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        return Self::rgba_u8((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8,);
    }
}

#[derive(Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<Pixel>,

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
            pixels: vec![Pixel::rgba_f32(1.0, 1.0, 1.0, 1.0); width * height],
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
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Pixel) {
        // if x < self.width && y < self.height {
        // }
        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    // Get the color of a specific pixel
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Pixel> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.pixels[index])
        } else {
            None
        }
    }

    // Fill the canvas with a specific color
    pub fn fill(&mut self, color: Pixel) {
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

            let first_dot = Xy::new(first_dot.x as usize, self.height - (first_dot.y as usize));
            // let second_dot = Xy::new(second_dot.x as usize, self.height - (second_dot.y as usize));

            let diameter: isize = 20;
            let radius = (diameter - 1)/2;
            let radius_squared = radius * radius;

            let (x, y) = (first_dot[X] as isize, first_dot[Y] as isize);
            for dx in (-radius)..radius {
                for dy in (-radius)..radius {
                    if dx * dx + dy * dy <= radius_squared {
                        let x = max(x - dx, 0) as usize;
                        let y = max(y - dy, 0) as usize;
                        self.set_pixel(x, y, Pixel::rgba_u8(0, 0, 0, 255))
                    }
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
