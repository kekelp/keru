use crate::*;

pub(crate) struct TextRenderer {
    pub(crate) device: Device,
    pub(crate) queue: Queue,

    // Combined texture arrays and single bind group
    pub(crate) mask_texture_array: Texture,
    pub(crate) color_texture_array: Texture,
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,

    pub(crate) sampler: Sampler,
    pub(crate) params_buffer: Buffer,

    pub(crate) pipeline: RenderPipeline,
    pub(crate) atlas_size: u32,

    pub(crate) srgb: bool,
}

impl Text {
    /// Returns the source of the composable text shader.
    pub fn composable_shader_source() -> &'static str {
        include_str!("shaders/keru_text.slang")
    }

    /// Get the bind group for external rendering.
    pub fn bind_group(&self) -> BindGroup {
        self.renderer.bind_group.clone()
    }

    /// Get the bind group layout for external rendering.
    pub fn bind_group_layout(&self) -> BindGroupLayout {
        self.renderer.bind_group_layout.clone()
    }
}

impl TextRenderer {
    /// Render all prepared text using the provided render pass.
    pub fn render(&self, pass: &mut RenderPass, render_data: &RenderData) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, render_data.glyph_quads.buffer().slice(..));

        let total_instances = render_data.glyph_quads.len();

        if total_instances > 0 {
            // Single draw call for all instances
            pass.draw(0..4, 0..(total_instances as u32));
        }
    }
}

impl TextRenderer {
    pub(crate) fn rebuild_texture_arrays(&mut self, render_data: &mut RenderData) {
        let (mask_texture_array, color_texture_array) = rebuild_texture_arrays(
            &self.device,
            &self.queue,
            self.atlas_size,
            &mut render_data.mask_atlas_pages,
            &mut render_data.color_atlas_pages,
            self.srgb,
        );

        self.mask_texture_array = mask_texture_array;
        self.color_texture_array = color_texture_array;
        // Bind group recreation is handled by the caller (load_to_gpu)
    }

    pub(crate) fn update_texture_arrays(&mut self, render_data: &mut RenderData) {
        for (i, page) in render_data.mask_atlas_pages.iter_mut().enumerate() {
            if page.needs_upload {
                let raw = page.image.as_raw();
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.mask_texture_array,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    raw,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(page.image.width()),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: page.image.width(),
                        height: page.image.height(),
                        depth_or_array_layers: 1,
                    },
                );
                #[cfg(debug_assertions)]
                {
                    render_data.stats.gpu_bytes += raw.len() as u64;
                }
                page.needs_upload = false;
            }
        }

        // Update only dirty color texture array pages
        for (i, page) in render_data.color_atlas_pages.iter_mut().enumerate() {
            if page.needs_upload {
                let raw = page.image.as_raw();
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.color_texture_array,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    raw,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(page.image.width() * 4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: page.image.width(),
                        height: page.image.height(),
                        depth_or_array_layers: 1,
                    },
                );
                #[cfg(debug_assertions)]
                {
                    render_data.stats.gpu_bytes += raw.len() as u64;
                }
                page.needs_upload = false;
            }
        }
    }


    pub(crate) fn recreate_bind_group(&mut self, render_data: &RenderData) {
        self.bind_group = create_bind_group(
            &self.device,
            &self.mask_texture_array,
            &self.color_texture_array,
            &render_data.glyph_quads,
            &self.sampler,
            &self.params_buffer,
            &render_data.box_data,
            &render_data.group_transforms,
            &self.bind_group_layout,
        );
    }
}
