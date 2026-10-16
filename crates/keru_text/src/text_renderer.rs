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
    pub(crate) staging_buffer: Option<Buffer>,

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
    pub(crate) fn upload_atlas(&mut self, render_data: &mut RenderData) -> bool {
        let n_mask_pages = render_data.mask_atlas_pages.len() as u32;
        let n_color_pages = render_data.color_atlas_pages.len() as u32;
        let old_mask_layers = self.mask_texture_array.depth_or_array_layers();
        let old_color_layers = self.color_texture_array.depth_or_array_layers();

        let needs_growth = n_mask_pages > old_mask_layers || n_color_pages > old_color_layers;
        let staging = &mut render_data.glyph_staging_image;

        if !needs_growth && staging.uploads.is_empty() {
            return false;
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if n_mask_pages > old_mask_layers {
            let new_texture = create_mask_texture_array(&self.device, self.atlas_size, n_mask_pages);
            copy_layers(&mut encoder, &self.mask_texture_array, &new_texture, self.atlas_size, old_mask_layers);
            self.mask_texture_array = new_texture;
        }

        if n_color_pages > old_color_layers {
            let new_texture = create_color_texture_array(&self.device, self.atlas_size, n_color_pages, self.srgb);
            copy_layers(&mut encoder, &self.color_texture_array, &new_texture, self.atlas_size, old_color_layers);
            self.color_texture_array = new_texture;
        }

        if !staging.uploads.is_empty() {
            let needed_size = staging.data.len() as u64;
            let too_small = self.staging_buffer.as_ref().is_none_or(|buffer| buffer.size() < needed_size);
            if too_small {
                self.staging_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("glyph staging buffer"),
                    size: needed_size.max(STAGING_BASE_CAPACITY as u64).next_power_of_two(),
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            let staging_buffer = self.staging_buffer.as_ref().unwrap();

            self.queue.write_buffer(staging_buffer, 0, &staging.data);

            for upload in &staging.uploads {
                let texture = match upload.content_type {
                    Content::Mask => &self.mask_texture_array,
                    Content::Color => &self.color_texture_array,
                    Content::SubpixelMask => unreachable!(),
                };

                encoder.copy_buffer_to_texture(
                    wgpu::TexelCopyBufferInfo {
                        buffer: staging_buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: upload.staging_offset,
                            bytes_per_row: Some(staging.stride),
                            rows_per_image: None,
                        },
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: upload.x, y: upload.y, z: upload.page },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: upload.width,
                        height: upload.height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            #[cfg(debug_assertions)]
            {
                render_data.stats.gpu_bytes += needed_size;
            }

            staging.clear();
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        if self.staging_buffer.as_ref().is_some_and(|buffer| buffer.size() > STAGING_BASE_CAPACITY as u64) {
            self.staging_buffer = None;
        }

        needs_growth
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

/// Copy the first `layers` layers of `src` into `dst`.
fn copy_layers(encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, dst: &wgpu::Texture, atlas_size: u32, layers: u32) {
    encoder.copy_texture_to_texture(
        src.as_image_copy(),
        dst.as_image_copy(),
        wgpu::Extent3d {
            width: atlas_size,
            height: atlas_size,
            depth_or_array_layers: layers,
        },
    );
}
