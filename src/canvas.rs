use std::{cmp::max, mem::size_of};
use wgpu::*;

use bytemuck::{Pod, Zeroable};
use glam::*;

use {BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, Buffer, ColorTargetState, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, RenderPass, RenderPipeline, Texture, TextureAspect};
use winit::{dpi::PhysicalPosition, event::{ElementState, Event, MouseButton, WindowEvent}, keyboard::{Key, ModifiersState, NamedKey}};

use crate::{ui::Xy, Scale, SWAPCHAIN_FORMAT};

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

pub struct Canvas {
    pub width: usize,
    pub height: usize,

    pub image_width: usize,
    pub image_height: usize,
    pub pixels: Vec<PixelColor>,

    pub scale: DVec2,
    pub rotation: EpicRotation,

    // this translation is in screen pixels right now I think
    pub translation: DVec2,

    pub backups: Vec<Vec<PixelColor>>,
    pub backups_i: usize,
    pub need_backup: bool,

    pub space: bool,
    pub clicking: bool,

    pub mouse_dots: Vec<PhysicalPosition<f64>>,
    pub end_stroke: bool,

    // todo: doesn't UI also keep this? maybe its good to keep them separately doe
    pub last_mouse_pos: PhysicalPosition<f64>,

    pub needs_sync: bool,
    pub needs_render: bool,

    pub texture: Texture,
    pub render_pipeline: RenderPipeline,
    pub canvas_bind_group: BindGroup,
    pub canvas_uniform_buffer: Buffer,

    pub is_drawing: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    image_size: [f32; 4],
    transform: [[f32; 4]; 4],
}

impl Canvas {
    // Create a new canvas with the given width and height, initialized to a background color
    pub fn new(width: usize, height: usize, device: &Device, queue: &Queue, base_uniforms: &Buffer) -> Self {
        // default transformations
        let scale = dvec2(1.0, 1.0);
        let rotation = EpicRotation::new(-0.0_f64.to_radians());
        let translation = dvec2(0.0, 0.0);

        let (image_width, image_height) = (width.scale(0.8), height.scale(0.8));
        
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Canvas Texture"),
            size: Extent3d {
                width: image_width as u32,
                height: image_height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let canvas_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(size_of::<CanvasUniforms>() as u64),
                    },
                    count: None,
                }
            ],
        });
        
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        let canvas_uniform_buffer = device.create_buffer(
            &BufferDescriptor {
                label: Some("Canvas Uniform Buffer"),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                size: size_of::<CanvasUniforms>() as u64,
                mapped_at_creation: false,
            }
        );

        let canvas_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &canvas_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: base_uniforms.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: canvas_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Canvas Bind Group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&canvas_bind_group_layout],
            push_constant_ranges: &[],
        });
    
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: SWAPCHAIN_FORMAT,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });
        
        let mut canvas = Canvas {
            width,
            height,
            image_width,
            image_height,
            pixels: vec![PixelColor::rgba_f32(1.0, 1.0, 1.0, 1.0); image_width * image_height],
            backups: Vec::new(),
            backups_i: 0,

            scale,
            rotation,
            translation,
            canvas_uniform_buffer,

            texture,
            render_pipeline,
            canvas_bind_group,
            need_backup: true,
            space: false,
            clicking: false,

            last_mouse_pos: PhysicalPosition::default(),

            mouse_dots: Vec::new(),
            end_stroke: false,

            needs_sync: true,
            needs_render: true,
            is_drawing: false,
        };

        // fill with test colors
        // for x in 0..width {
        //     for y in 0..height {
        //         if let Some(pixel) = canvas.get_pixel(x, y) {
        //             *pixel = PixelColor::rgba_f32(x as f32 / width as f32, 0.0, y as f32 / height as f32, 1.0);
        //         }
                
        //     }
        // }

        canvas.update_shader_transform(queue);

        return canvas;
    }

    pub fn update_shader_transform(&mut self, queue: &Queue) {
        let aspect = self.height as f32 / self.width as f32;
        let scale_x = self.scale.x as f32; 
        let scale_y = self.scale.y as f32 / aspect; 
        let mat_scale = Mat4::from_scale(vec3(scale_x, scale_y, 1.0));
        let mat_rotation = Mat4::from_rotation_z(self.rotation.angle() as f32);

        // scale with the weird aspect or something
        let scaled_translation = self.translation / self.width as f64 * 2.0;
        let mat_translation = Mat4::from_translation(
            vec3(
                scaled_translation.x as f32,
                - scaled_translation.y as f32,
                1.0
            )
        );
                       
        let transform = mat_scale * mat_translation * mat_rotation;

        let canvas_uniforms = CanvasUniforms {
            image_size: [self.image_width as f32, self.image_height as f32, 0.0, 0.0],
            transform: transform.to_cols_array_2d(),
        };

        queue.write_buffer(
            &self.canvas_uniform_buffer,
            0,
            &bytemuck::bytes_of(&canvas_uniforms)[..size_of::<CanvasUniforms>()],
        );
    }

    // Set a pixel to a specific color
    pub fn paint_pixel(&mut self, x: usize, y: usize, paint_color: PixelColorF32, brush_alpha: f32) {

        if let Some(old_color) = self.get_pixel(x, y) {
            let old_color_f32 = old_color.to_f32s();
            
            if brush_alpha > 0.0 {
                
                
                let new_color = PixelColorF32 {
                    
                    r: old_color_f32.r * (1.0 - brush_alpha) + paint_color.r * (brush_alpha),
                    g: old_color_f32.g * (1.0 - brush_alpha) + paint_color.g * (brush_alpha),
                    b: old_color_f32.b * (1.0 - brush_alpha) + paint_color.b * (brush_alpha),
                    
                    a: 1.0,
                };
                
                
                *old_color = new_color.to_u8s();
            }
        }
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> Option<&mut PixelColor> {
        // let y = self.image_height - y;

        if x < self.image_width && y < self.image_height {
            let index = y * self.image_width + x;
            Some(&mut self.pixels[index])
        } else {
            None
        }
    }

    pub fn center_screen_coords(&self, p: DVec2) -> DVec2 {
        // todo, use a dvec2 directly in self?
        let w = self.width as f64;
        let h = self.height as f64;
        let screen_size = dvec2(w, h);
        
        return p - screen_size / 2.0;
    }

    pub fn decenter_screen_coords(&self, p: DVec2) -> DVec2 {
        // todo, use a dvec2 directly in self?
        let w = self.width as f64;
        let h = self.height as f64;
        let screen_size = dvec2(w, h);
        
        return p + screen_size / 2.0;
    }

    pub fn center_image_coords(&self, p: DVec2) -> DVec2 {
        let w = self.image_width as f64;
        let h = self.image_height as f64;
        let image_size = dvec2(w, h);

        return p - image_size/2.0;
    }

    pub fn decenter_image_coords(&self, p: DVec2) -> DVec2 {
        let w = self.image_width as f64;
        let h = self.image_height as f64;
        let image_size = dvec2(w, h);

        return p + image_size/2.0;
    }

    pub fn screen_to_image(&self, x: f64, y: f64) -> (f64, f64) {
        let p = dvec2(x, y);

        let p = self.center_screen_coords(p);

        // apply the canvas transforms to convert from centered screen pixels to centered image pixels
        let p = p / self.scale;
        let p = p - self.translation;
        let mut p = p.rotate(self.rotation.vec());

        // invert y
        p.y = - p.y;

        let p = self.decenter_image_coords(p);

        return (p.x, p.y);
    }

    pub fn image_to_screen(&self, x: f64, y: f64) -> DVec2 {
        let mut p = dvec2(x, y);
    
        p = self.center_image_coords(p);
    
        // invert y
        p.y = -p.y;
    
        p = p.rotate(-self.rotation.vec());
        p = p + self.translation;
        p = p * self.scale;
    
        p = self.decenter_screen_coords(p);
    
        return p;
    }
    


    pub fn draw_dots(&mut self) {
        if self.mouse_dots.len() == 0 {
            return;
        }

        if self.mouse_dots.len() == 1 {
            let (x,y) = self.screen_to_image(self.mouse_dots[0].x, self.mouse_dots[0].y);

            self.draw_circle(x as isize, y as isize);
            
            return;
        }
        
        for i in 0..(self.mouse_dots.len() - 1) {

            let (x,y) = self.screen_to_image(self.mouse_dots[i].x, self.mouse_dots[i].y);
            let first_dot = Xy::new(x as f64,y as f64);

            let (x,y) = self.screen_to_image(self.mouse_dots[i + 1].x, self.mouse_dots[i + 1].y);
            let second_dot = Xy::new(x as f64,y as f64);

            let first_center = Xy::new(first_dot.x, first_dot.y);
            let second_center = Xy::new(second_dot.x, second_dot.y);
            
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
                    bytes_per_row: Some(self.image_width as u32 * 4),
                    rows_per_image: None,
                },
                Extent3d {
                    width: self.image_width as u32,
                    height: self.image_height as u32,
                    depth_or_array_layers: 1,
                },
            );
            self.needs_sync = false;
        // }

        // if self.needs_render {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.canvas_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
            
            self.needs_render = false;
        // }
    }

    pub fn handle_events(&mut self, full_event: &winit::event::Event<()>, key_mods: &ModifiersState, queue: &Queue) {
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Left {

                        self.is_drawing = *state == ElementState::Pressed;
                        if ! self.space {

                            self.mouse_dots.push(self.last_mouse_pos);
                            
                            // do this on release so that it doesn't get in the way computationally speaking
                            if *state == ElementState::Released {
                                self.end_stroke = true;
                                self.need_backup = true;
                            }
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    self.last_mouse_pos = *position;

                    if self.is_drawing && ! self.space {
                        self.mouse_dots.push(*position);
                    }
                },
                WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                    // println!("  {:?}", event );
                    if ! is_synthetic && event.state.is_pressed() {
                        match &event.logical_key {
                            Key::Character(new_char) => {
                                match new_char.as_str() {
                                    "z" => {
                                        if key_mods.control_key() {
                                            self.undo();
                                        }
                                    },
                                    "Z" => {
                                        if key_mods.control_key() {
                                            self.redo();
                                        }
                                    },
                                        _ => {},
                                    }
                                }
                                _ => {}
                            }
                    }

                    if ! is_synthetic {
                        match &event.logical_key {
                            Key::Named(NamedKey::Space) => {
                                self.space = event.state.is_pressed();
                            }
                            _ => {},
                        }
                    }
                },
                // todo, this sucks actually.
                WindowEvent::Resized(size) => {
                    self.width = size.width as usize;
                    self.height = size.height as usize;
                    self.update_shader_transform(&queue);
                },

                _ => {}
            }
        }
    }

    pub fn push_backup(&mut self) {
        self.backups.truncate(self.backups_i);
        self.backups.push(self.pixels.clone());

        self.backups_i += 1;
    }

    pub fn undo(&mut self) {
        if self.backups_i >= 2 {
            self.backups_i -= 1;
            self.pixels = self.backups[self.backups_i - 1].clone();

        }
    }

    pub fn redo(&mut self) {
        if self.backups_i < self.backups.len() {
            self.backups_i += 1;
            self.pixels = self.backups[self.backups_i - 1].clone();
        }
    }

}


pub trait ReasonableRotation {
    fn rotated(self, rhs: f64) -> Self;
}

impl ReasonableRotation for DVec2 {
    fn rotated(self, rhs: f64) -> Self {
        let cos = rhs.cos();
        let sin = rhs.sin();
        return Self {
            x: self.x * cos - self.y * sin,
            y: self.y * cos + self.x * sin,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EpicRotation {
    angle: f64,
    vec: DVec2,
}
impl EpicRotation {
    pub fn new(angle_radians: f64) -> Self {
        return Self {
            angle: angle_radians,
            vec: dvec2(angle_radians.cos(), angle_radians.sin()),
        }
    }
    pub fn angle(&self) -> f64 {
        return self.angle;
    }
    pub fn vec(&self) -> DVec2 {
        return self.vec;
    }

    pub fn inverse_vec(&self) -> DVec2 {
        return dvec2(self.vec.x, -self.vec.y);
    }

    pub fn cos(&self) -> f64 {
        return self.vec.x;
    }
    pub fn sin(&self) -> f64 {
        return self.vec.y;
    }
}
