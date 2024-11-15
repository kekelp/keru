use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

// Struct that holds the render pipeline and a buffer for rectangle vertices
pub struct ColorPicker {
    vertex_buffer: Buffer,
    render_pipeline: RenderPipeline,
    coords: [f32; 4],
}

impl ColorPicker {
    pub fn new(device: &Device) -> Self {
        // Define the rectangle's vertices based on the input coordinates
        // This will define the four corners of the rectangle
        let coords = [0.0, 0.0, 0.2, 0.2];
        
        // Vertex buf
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Color Picker Rectangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&Self::vertices_from_coords(coords)),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vertex_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array!( 0 => Float32x2 ),
        };

        // Shader        
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shaders/color_picker.wgsl").into()),
        });
        
        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
            render_pipeline,
            coords,
        }
    }

    pub fn render<'pass>(&'pass mut self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..4, 0..1);
    }

    fn vertices_from_coords(coords: [f32; 4]) -> [[f32; 2]; 4] {
        return [
            [coords[0], coords[1]], // Bottom-left
            [coords[2], coords[1]], // Bottom-right
            [coords[0], coords[3]], // Top-left
            [coords[2], coords[3]], // Top-right
        ];
    }

    pub fn update_coordinates(&self, queue: &Queue) {
        let vertices = Self::vertices_from_coords(self.coords);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }
}
