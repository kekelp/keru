use basic_window_loop::basic_depth_stencil_state;
use basic_window_loop::Context;
use blue::*;

use blue::XyRect;

use bytemuck::{Pod, Zeroable};
use wgpu::*;

use crate::color_picker::*;

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

impl ColorPickerRenderer {
    pub fn new(ctx: &Context, base_uniforms: &Buffer) -> Self {
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
            depth_stencil: Some(basic_depth_stencil_state()),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buffer,
            bind_group,
            render_pipeline,
        }
    }

}

