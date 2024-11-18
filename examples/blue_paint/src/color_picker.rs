use blue::basic_window_loop::Context;
use blue::XyRect;
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

// Struct that holds the render pipeline and a buffer for rectangle vertices
pub struct ColorPicker {
    vertex_buffer: Buffer,
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,
    pub coords: [XyRect; 1],
}

impl ColorPicker {
    pub fn new(ctx: &Context, base_uniforms: &Buffer) -> Self {
        // Define the rectangle's vertices based on the input coordinates
        // This will define the four corners of the rectangle
        let coords = [XyRect::new_symm([0.0, 0.9])];

        let vertex_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 4]>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array!( 0 => Float32x2, 1 => Float32x2 ),
        };

        // Vertex buf
        let vertex_buffer = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Color Picker Rectangle Vertex Buffer"),
            contents: &bytemuck::cast_slice(&[0.0; 4]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
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
                entry_point: "vs_main",
                buffers: &[vertex_layout],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(BlendState::REPLACE),
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
            coords,
        }
    }

    pub fn render<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }

    pub fn update_coordinates(&self, queue: &Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.coords));
    }
}
