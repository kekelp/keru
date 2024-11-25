use basic_window_loop::Context;
use blue::*;

use blue::XyRect;

use bytemuck::{Pod, Zeroable};
use wgpu::*;

use crate::color_picker::*;

/// A struct with the information needed to render an ui rectangle on the screen.
/// Despite the name, it is also used for checking for click resolution.
/// The Ui state keeps a Vec of these.
#[repr(C)]
#[derive(Default, Debug, Pod, Zeroable, Copy, Clone)]
pub(crate) struct ColorPickerRenderRect {
    pub rect: XyRect,
    pub z: f32,
    pub hcl_color: [f32; 3],
}

impl ColorPickerRenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 4] {
        vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32,
            3 => Float32x3,
        ]
    }
}

impl ColorPicker {
    pub fn new(ctx: &Context, base_uniforms: &Buffer) -> Self {
        // Define the rectangle's vertices based on the input coordinates
        // This will define the four corners of the rectangle
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<ColorPickerRenderRect>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ColorPickerRenderRect::buffer_desc(),
        };

        // Vertex buf
        let vertex_buffer = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("Color Picker Rectangle Vertex Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size: (size_of::<ColorPickerRenderRect>() * 2) as u64,
            mapped_at_creation: false,
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            ],
        });
        
        let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: base_uniforms.as_entire_binding(),
                },
            ],
            label: Some("Color Picker Bind Group"),
        });

        // Shader        
        let shader = ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shaders/color_picker.wgsl").into()),
        });
        
        // Pipeline
        let pipeline_layout = ctx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Color Picker Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = ctx.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Color Picker Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            bind_group,
            render_pipeline,
            hcl_color: HclColor {
                hue: 0.3,
                saturation: 0.5,
                brightness: 0.5,
            }
        }
    }

    pub fn render<'pass>(&mut self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..2);
    }

    pub fn prepare(&self, ui: &mut Ui, queue: &wgpu::Queue) -> Option<()> {
        let wheel_rect = ColorPickerRenderRect {
            rect: ui.get_node(OKLAB_HUE_WHEEL)?.render_rect(),
            z: 0.0,
            hcl_color: self.hcl_color.into(),
        };

        let square_rect = ColorPickerRenderRect {
            rect: ui.get_node(OKLAB_SQUARE)?.render_rect(),
            z: 0.0,
            hcl_color: self.hcl_color.into(),
        };

        // to keep the rust-side boilerplate to a minimum, we use the same pipeline for all rects (wheel and main square) and have the shader do different things based on the instance index.
        // this means that the order here matters.
        let coords = [wheel_rect, square_rect];

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&coords));

        return Some(());
    }
}
