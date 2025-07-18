use std::{cmp::max, mem::size_of};
use wgpu::*;

use bytemuck::{Pod, Zeroable};
use glam::*;

use {BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, Buffer, ColorTargetState, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, RenderPass, RenderPipeline, Texture, TextureAspect};
use winit::dpi::PhysicalPosition;

use keru::{basic_window_loop::{basic_depth_stencil_state, Context}, winit_key_events::KeyInput, winit_mouse_events::MouseInput, Xy};

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl PixelColor {
    pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        return Self { r, g, b, a }
    }

    pub const fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        return Self::rgba_u8((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8,);
    }

    pub const fn to_f32s(self) -> PixelColorF32 {
        return PixelColorF32 {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: self.a as f32 / 255.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PixelColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        return PixelColorF32 {
            r,
            g,
            b,
            a,
        }
    }

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

pub struct Canvas {
    pub scroll: DVec2,

    pub mouse_input: MouseInput<()>,
    pub key_input: KeyInput,

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
    pub need_rerender: bool,

    pub texture: Texture,
    pub render_pipeline: RenderPipeline,
    pub canvas_bind_group: BindGroup,
    pub canvas_uniform_buffer: Buffer,

    pub is_drawing: bool,
    pub radius: f64,

    pub eraser_mode: bool,
    pub paint_color: PixelColorF32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CanvasUniforms {
    image_size: [f32; 4],
    transform: [[f32; 4]; 4],
}

impl Canvas {
    pub fn new(ctx: &Context, base_uniforms: &Buffer) -> Self {
        // default transformations
        let scale = dvec2(1.0, 1.0);
        let rotation = EpicRotation::new(-0.0_f64.to_radians());
        let translation = dvec2(0.0, 0.0);

        let (width, height) = (ctx.width() as usize, ctx.height() as usize);
        let (image_width, image_height) = (width.scale(0.8), height.scale(0.8));
        
        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("Canvas Texture"),
            size: Extent3d {
                width: image_width as u32,
                height: image_height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let canvas_bind_group_layout = ctx.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
        let sampler = ctx.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        
        let canvas_uniform_buffer = ctx.device.create_buffer(
            &BufferDescriptor {
                label: Some("Canvas Uniform Buffer"),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                size: size_of::<CanvasUniforms>() as u64,
                mapped_at_creation: false,
            }
        );

        let canvas_bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
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

        let render_pipeline_layout = ctx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Canvas Render Pipeline Layout"),
            bind_group_layouts: &[&canvas_bind_group_layout],
            push_constant_ranges: &[],
        });
    
        let shader = ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shaders/canvas.wgsl").into()),
        });

        let render_pipeline = ctx.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Canvas Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: ctx.surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(basic_depth_stencil_state()),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        
        let mut canvas = Canvas {
            mouse_input: MouseInput::default(),
            key_input: KeyInput::default(),

            scroll: dvec2(0.0, 0.0),
            
            // input: WinitInputHelper::new(),
            width,
            height,
            image_width,
            image_height,
            pixels: vec![PixelColor::rgba_f32(1.0, 1.0, 1.0, 1.0); image_width * image_height],
            backups: Vec::with_capacity(20),
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

            mouse_dots: Vec::with_capacity(100),
            end_stroke: false,

            needs_sync: true,
            need_rerender: true,
            is_drawing: false,

            radius: 5.0,
            eraser_mode: false,
            paint_color: PixelColorF32::new(0.2, 0.8, 0.2, 1.0),

        };

        // fill with test colors
        // for x in 0..width {
        //     for y in 0..height {
        //         if let Some(pixel) = canvas.get_pixel(x, y) {
        //             *pixel = PixelColor::rgba_f32(x as f32 / width as f32, 0.0, y as f32 / height as f32, 1.0);
        //         }
                
        //     }
        // }

        canvas.update_shader_transform(&ctx.queue);

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

        let erase = self.eraser_mode;

        if let Some(old_color) = self.get_pixel(x, y) {
            let old_color_f32 = old_color.to_f32s();
            
            if brush_alpha > 0.0 {
                
                if ! erase {

                    let new_color = PixelColorF32 {
                        
                        r: old_color_f32.r * (1.0 - brush_alpha) + paint_color.r * (brush_alpha),
                        g: old_color_f32.g * (1.0 - brush_alpha) + paint_color.g * (brush_alpha),
                        b: old_color_f32.b * (1.0 - brush_alpha) + paint_color.b * (brush_alpha),
                        
                        a: 1.0,
                    };
                    
                    *old_color = new_color.to_u8s();   
                } else {
                    *old_color = PixelColor::rgba_f32(1.0, 1.0, 1.0, 1.0);
                }
            }
        }

        self.need_rerender = true;
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> Option<&mut PixelColor> {
        if x < self.image_width && y < self.image_height {
            let index = y * self.image_width + x;
            Some(&mut self.pixels[index])
        } else {
            None
        }
    }
    pub fn get_pixel_nonmut(&self, x: usize, y: usize) -> Option<&PixelColor> {
        if x < self.image_width && y < self.image_height {
            let index = y * self.image_width + x;
            Some(&self.pixels[index])
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
        p += self.translation;
        p *= self.scale;
    
        p = self.decenter_screen_coords(p);
    
        return p;
    }
    


    pub fn draw_dots(&mut self) {
        if self.mouse_dots.is_empty() {
            return;
        }

        if self.mouse_dots.len() == 1 {
            let (x,y) = self.screen_to_image(self.mouse_dots[0].x, self.mouse_dots[0].y);

            self.draw_circle(x as isize, y as isize);
            
            return;
        }
        
        for i in 0..(self.mouse_dots.len() - 1) {

            let (x,y) = self.screen_to_image(self.mouse_dots[i].x, self.mouse_dots[i].y);
            let first_dot = Xy::new(x,y);

            let (x,y) = self.screen_to_image(self.mouse_dots[i + 1].x, self.mouse_dots[i + 1].y);
            let second_dot = Xy::new(x,y);

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
        let radius = self.radius;
        let pixel_radius = (radius as isize) + 2; // some more pixels for antialiasing? 

        for dx in (-pixel_radius)..pixel_radius {
            for dy in (-pixel_radius)..pixel_radius {
                let pixel_x = max(x0 - dx, 0) as usize;
                let pixel_y = max(y0 - dy, 0) as usize;
                let pixel = Xy::new(pixel_x, pixel_y);
                let center = Xy::new(pixel_x as f64, pixel_y as f64);

                let pos = center + (dx as f64, dy as f64);

                let alpha = radius - ((center - pos).x.powi(2) + (center - pos).y.powi(2)).sqrt();
                let alpha = alpha.clamp(0.0, 1.0);

                self.paint_pixel(pixel.x, pixel.y, self.paint_color, alpha as f32);
            }
        }
    }

    pub fn pixel_info(&self) -> Option<PixelInfo> {
        let (x, y) = (self.last_mouse_pos.x, self.last_mouse_pos.y);
        let (x, y) = self.screen_to_image(x, y);
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (x, y) = (x as u32, y as u32);
        let color = self.get_pixel_nonmut(x as usize, y as usize)?.to_f32s();
    
        return Some(PixelInfo { x, y, color }); 
    }

    // todo, do we really believe in this prepare/render stuff? canvas was writing the texture in its render() and it was fine.
    pub fn prepare(&mut self, queue: &Queue, ) {
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
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.canvas_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        self.need_rerender = false;
    }

    pub fn needs_rerender(&self) -> bool {
        return self.need_rerender
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
            
            self.need_rerender = true;
        }
    }

    pub fn redo(&mut self) {
        if self.backups_i < self.backups.len() {
            self.backups_i += 1;
            self.pixels = self.backups[self.backups_i - 1].clone();

            self.need_rerender = true;
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


pub struct PixelInfo {
    pub x: u32,
    pub y: u32,
    pub color: PixelColorF32,
}

pub trait Scale {
    fn scale(self, scale: f32) -> Self;
}
impl Scale for usize {
    fn scale(self, scale: f32) -> Self {
        return (self as f32 * scale) as Self;
    }
}
impl Scale for u32 {
    fn scale(self, scale: f32) -> Self {
        return (self as f32 * scale) as Self;
    }
}