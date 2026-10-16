use crate::*;

pub(crate) const STAGING_BASE_CAPACITY: usize = 512 * 1024;

/// CPU-side staging image for new glyphs
pub(crate) struct GlyphStagingImage {
    pub(crate) stride: u32,
    pub(crate) data: Vec<u8>,
    pub(crate) uploads: Vec<GlyphUpload>,

    shelf_y: u32,
    shelf_height: u32,
    shelf_cursor: u32,
}

pub(crate) struct GlyphUpload {
    pub(crate) content_type: Content,
    pub(crate) page: u32,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) staging_offset: u64,
}

impl GlyphStagingImage {
    pub(crate) fn new(atlas_size: u32) -> Self {
        let stride = (atlas_size * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        Self {
            stride,
            data: Vec::with_capacity(STAGING_BASE_CAPACITY),
            uploads: Vec::new(),
            shelf_y: 0,
            shelf_height: 0,
            shelf_cursor: 0,
        }
    }

    pub(crate) fn push(&mut self, content_type: Content, page: u32, x: u32, y: u32, width: u32, height: u32, pixels: &[u8]) {
        let bytes_per_pixel = match content_type {
            Content::Mask => 1,
            Content::Color => 4,
            Content::SubpixelMask => unreachable!(),
        };
        let row_bytes = width * bytes_per_pixel;

        if self.shelf_cursor + row_bytes > self.stride {
            self.shelf_y += self.shelf_height;
            self.shelf_height = 0;
            self.shelf_cursor = 0;
        }

        let staging_x = self.shelf_cursor;
        self.shelf_height = self.shelf_height.max(height);
        self.shelf_cursor = (self.shelf_cursor + row_bytes).next_multiple_of(4);

        let needed_len = ((self.shelf_y + self.shelf_height) * self.stride) as usize;
        if self.data.len() < needed_len {
            self.data.resize(needed_len, 0);
        }

        let stride = self.stride as usize;
        let row_bytes = row_bytes as usize;
        for row in 0..height as usize {
            let src = &pixels[(row * row_bytes)..((row + 1) * row_bytes)];
            let dst_start = (self.shelf_y as usize + row) * stride + staging_x as usize;
            self.data[dst_start..(dst_start + row_bytes)].copy_from_slice(src);
        }

        self.uploads.push(GlyphUpload {
            content_type,
            page,
            x,
            y,
            width,
            height,
            staging_offset: self.shelf_y as u64 * self.stride as u64 + staging_x as u64,
        });
    }

    pub(crate) fn clear(&mut self) {
        self.data.clear();
        self.data.shrink_to(STAGING_BASE_CAPACITY);
        self.uploads.clear();
        self.shelf_y = 0;
        self.shelf_height = 0;
        self.shelf_cursor = 0;
    }
}
