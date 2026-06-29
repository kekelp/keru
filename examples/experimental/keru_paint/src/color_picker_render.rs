use crate::window::Context;
use keru::XyRect;
use wgpu::*;

pub struct ColorPickerRenderer {
    pub pipeline: RenderPipeline,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    // Node rect in Keru graphics space (0..1): [min_x, min_y, max_x, max_y].
    rect: [f32; 4],
    // hue (radians), chroma, lightness
    hcl: [f32; 3],
    // window size in pixels, so the shader can size the ring in real pixels.
    window_size: [f32; 2],
    // 0.0 = hue wheel, 1.0 = lightness/chroma square.
    mode: f32,
}

impl ColorPickerRenderer {
    pub fn new(ctx: &Context) -> Self {
        let shader = ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Color Picker Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/color_picker.wgsl").into()),
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Color Picker Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX_FRAGMENT,
                range: 0..(size_of::<PushConstants>() as u32),
            }],
        });

        let pipeline = ctx.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Color Picker Pipeline"),
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
                    format: ctx.surface_config.format,
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

        Self { pipeline }
    }

    pub fn draw(
        &self,
        render_pass: &mut RenderPass,
        rect: XyRect,
        hcl: [f32; 3],
        window_size: [f32; 2],
        mode: u32,
    ) {
        let push_constants = PushConstants {
            rect: [rect.x[0], rect.y[0], rect.x[1], rect.y[1]],
            hcl,
            window_size,
            mode: mode as f32,
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::bytes_of(&push_constants),
        );
        render_pass.draw(0..4, 0..1);
    }
}
