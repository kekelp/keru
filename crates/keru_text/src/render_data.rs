use crate::*;

/// Statistics about work done during a render cycle.
///
/// Call [`Text::render_stats()`] after [`Text::prepare_all()`] and [`Text::load_to_gpu()`] to see
/// what work was done and whether the optimizations are working.
///
/// Only available in debug builds.
#[cfg(debug_assertions)]
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Number of glyphs rasterized (cache misses that needed CPU rasterization work).
    pub glyphs_rasterized: u32,

    /// Number of glyph quads built and added to the buffer.
    pub glyph_quads_built: u32,

    /// Total bytes written to GPU (buffers + textures).
    pub gpu_bytes: u64,

    /// Number of text boxes that were fully relaid out this frame.
    pub relayouts: u32,

    /// Number of text boxes prepared (not hidden/offscreen) this frame.
    pub boxes_prepared: u32,

    /// Number of text boxes skipped (hidden or offscreen) this frame.
    pub boxes_skipped: u32,
}

pub(crate) struct RenderData {
    pub(crate) frame: u64,
    pub(crate) tmp_image: Image,

    pub(crate) glyph_cache: LruCache<GlyphKey, Option<StoredGlyph>, BuildHasherDefault<FxHasher>>,
    pub(crate) last_frame_evicted: u64,

    pub(crate) mask_atlas_pages: Vec<AtlasPage<GrayImage>>,
    pub(crate) color_atlas_pages: Vec<AtlasPage<RgbaImage>>,

    pub(crate) glyph_quads: GpuHeap<GlyphQuad>,
    pub(crate) box_data: GpuSlab<BoxGpu>,
    pub(crate) group_transforms: GpuSlab<GroupTransform>,

    pub(crate) params: Params,
    pub(crate) atlas_size: u32,

    pub(crate) needs_texture_array_rebuild: bool,
    pub(crate) needs_params_sync: bool,

    /// Handle into the glyph quad heap for the current frame's decoration quads (selections + cursor).
    pub(crate) decoration_quad_handle: Option<Handle>,

    pub(crate) scale_cx: Option<ScaleContext>,

    #[cfg(debug_assertions)]
    pub(crate) stats: RenderStats,
}

const CONTENT_TYPE_MASK: u32 = 0;
const CONTENT_TYPE_COLOR: u32 = 1;
const CONTENT_TYPE_DECORATION: u32 = 2;

// Scroll optimization tolerance: how many pixels we can scroll from the original
// preparation point before needing to reprepare (to ensure culled lines stay culled)
//
// This tolerance enables two optimizations to work together:
// 1. Line culling: Skip preparing lines outside the clip area (with tolerance margin)
// 2. Quad moving: When scrolling small distances, just adjust quad positions instead of repreparing
//
// The tolerance ensures that:
// - Lines are prepared with a margin (SCROLL_TOLERANCE) beyond the visible area
// - Quads can be moved within that margin without bringing unprepared lines into view
// - Once scrolling exceeds the tolerance, we reprepare everything with the new scroll offset
const SCROLL_TOLERANCE: f32 = 200.0;

/// Key for building a glyph cache
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct GlyphKey {
    /// Font ID
    pub font_id: u64,
    /// Glyph ID
    pub glyph_id: GlyphId,
    /// `f32` bits of font size
    pub font_size_bits: u32,
    /// Binning of fractional X offset
    pub x_bin: SubpixelBin::<4>,
    /// Binning of fractional Y offset
    pub y_bin: SubpixelBin::<4>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SubpixelBin<const N: u8>(pub u8);

fn quantize<const N: u8>(pos: f32) -> (i32, f32, SubpixelBin::<N>) {
    let trunc = pos as i32;
    let fract = pos - trunc as f32;

    let expanded_bin = if fract.is_sign_negative() {
        let abs_fract = fract.abs();
        ((2 * N) as f32 * (1.0 - abs_fract)).floor() as i32
    } else {
        ((2 * N) as f32 * fract).floor() as i32
    };

    let (adjusted_trunc, bin) = if expanded_bin >= 2 * N as i32 {
        (trunc + 1, 0)
    } else if expanded_bin <= 0 {
        (trunc, 0)
    } else {
        let compressed_bin = (expanded_bin + 1) / 2;
        (trunc, compressed_bin as u8)
    };

    // todo: return the fract rounded to the subpixel bin
    return (adjusted_trunc, fract, SubpixelBin::<N>(bin))
}

/// Per-text-box data stored in a separate buffer and referenced by index.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct BoxGpu {
    pub clip_rect_x: [f32; 2],            // 8 bytes - (x0, x1) in local space
    pub clip_rect_y: [f32; 2],            // 8 bytes - (y0, y1) in local space
    pub translation: [f32; 2],            // 8 bytes
    pub rotation: f32,                    // 4 bytes - rotation in radians
    pub scale: f32,                       // 4 bytes
    pub screen_clip_x: [f32; 2],          // 8 bytes - (min_x, max_x) in screen space
    pub screen_clip_y: [f32; 2],          // 8 bytes - (min_y, max_y) in screen space
    pub scroll_offset: [f32; 2],          // 8 bytes - scroll offset applied before transform
    pub slab_metadata: u32,               // 4 bytes
    pub depth: f32,                       // 4 bytes - z-order for rendering
    pub group_transform_index: u32,       // 4 bytes - index into group_transforms buffer
    pub opacity: f32,                     // 4 bytes - per-box opacity multiplier (1.0 = opaque)
    pub color_override: u32,              // 4 bytes. 0x000000 is "no override"
    pub _padding: u32,                    // 4 bytes
}

impl GpuSlabItem for BoxGpu {
    fn next_free(&self) -> Option<usize> {
        if self.slab_metadata == u32::MAX {
            None
        } else {
            Some(self.slab_metadata as usize)
        }
    }

    fn set_next_free(&mut self, i: Option<usize>) {
        match i {
            Some(i) => self.slab_metadata = i as u32,
            None => self.slab_metadata = u32::MAX,
        }
    }
}

/// The struct corresponding to the gpu-side representation of a text glyph.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct GlyphQuad {
    pub pos_packed: u32,                  // 4 bytes - pack x,y as u16,u16
    pub dim_packed: u32,                  // 4 bytes - pack width,height as u16,u16
    pub uv_origin_packed: u32,            // 4 bytes - pack u,v as u16,u16
    pub color: u32,                       // 4 bytes
    pub flags_and_page: u32,              // 4 bytes - flags (24 bits) + page_index (8 bits)
    pub box_index: u32,                   // 4 bytes - index into box_data array
    pub _padding1: u32,               // 8 bytes - padding for alignment
    pub _padding2: u32,               // 8 bytes - padding for alignment
}

// Helper functions to pack/unpack u16 pairs into u32
fn pack_u16_pair(a: u32, b: u32) -> u32 {
    (a & 0xFFFF) | ((b & 0xFFFF) << 16)
}

// Saturating cast from i32 to i16 (preserves negatives within i16 range)
fn saturating_i16_cast(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

// Pack i32 coordinates as i16 pairs with saturating cast (stored as u16 bit patterns)
fn pack_i32_pair_as_u16(a: i32, b: i32) -> u32 {
    let a_i16 = saturating_i16_cast(a);
    let b_i16 = saturating_i16_cast(b);
    (a_i16 as u16 as u32) | ((b_i16 as u16 as u32) << 16)
}

// Pack flags (24 bits) and page_index (8 bits) into u32
fn pack_flags_and_page(flags: u32, page_index: u32) -> u32 {
    (flags & 0xFFFFFF) | ((page_index & 0xFF) << 24)
}

/// Pack a color into the `0xRRGGBBAA` the shader unpacks.
pub(crate) fn pack_color_brush(color: ColorBrush) -> u32 {
    ((color.0[0] as u32) << 24)
    | ((color.0[1] as u32) << 16)
    | ((color.0[2] as u32) << 8)
    | (color.0[3] as u32)
}

fn create_box_data(clip_rect: Option<BoundingBox>, scroll_offset: (f32, f32), transform: Transform2D, screen_clip: Option<BoundingBox>, depth: f32, group_transform_index: u32, opacity: f32, color_override: Option<ColorBrush>) -> BoxGpu {
    // clip_rect from effective_clip_rect() is already in layout-local coordinates (includes scroll_offset)
    let (clip_rect_x, clip_rect_y) = if let Some(clip) = clip_rect {
        (
            [clip.x0 as f32, clip.x1 as f32],
            [clip.y0 as f32, clip.y1 as f32],
        )
    } else {
        (
            [f32::NEG_INFINITY, f32::INFINITY],
            [f32::NEG_INFINITY, f32::INFINITY],
        )
    };

    let (screen_clip_x, screen_clip_y) = match screen_clip {
        Some(clip) => ([clip.x0 as f32, clip.x1 as f32], [clip.y0 as f32, clip.y1 as f32]),
        None => ([f32::NEG_INFINITY, f32::INFINITY], [f32::NEG_INFINITY, f32::INFINITY]),
    };

    BoxGpu {
        clip_rect_x,
        clip_rect_y,
        translation: [transform.translation.0, transform.translation.1],
        rotation: transform.rotation,
        scale: transform.scale,
        screen_clip_x,
        screen_clip_y,
        scroll_offset: [scroll_offset.0, scroll_offset.1],
        slab_metadata: 0,
        depth,
        group_transform_index,
        opacity,
        // 0 is "no override": a fully transparent text color would be meaningless anyway.
        color_override: color_override.map(pack_color_brush).unwrap_or(0),
        _padding: 0,
    }
}

fn make_glyph_quad(glyph: &GlyphWithContext, stored_glyph: &StoredGlyph, box_index: u32) -> GlyphQuad {
    let y = glyph.quantized_pos_y - stored_glyph.placement_top as i32;
    let x = glyph.quantized_pos_x + stored_glyph.placement_left as i32;

    let (uv_x, uv_y) = (stored_glyph.alloc.rectangle.min.x, stored_glyph.alloc.rectangle.min.y);
    let (size_x, size_y) = (stored_glyph.size.width, stored_glyph.size.height);

    let (color, flags) = match stored_glyph.content_type {
        Content::Mask => (glyph.color, CONTENT_TYPE_MASK),
        Content::Color => (0xff_ff_ff_ff, CONTENT_TYPE_COLOR),
        Content::SubpixelMask => unreachable!(),
    };

    return GlyphQuad {
        pos_packed: pack_i32_pair_as_u16(x, y),
        dim_packed: pack_u16_pair(size_x as u32, size_y as u32),
        uv_origin_packed: pack_u16_pair(uv_x as u32, uv_y as u32),
        color,
        flags_and_page: pack_flags_and_page(flags, stored_glyph.page as u32),
        box_index,
        _padding1: 0,
        _padding2: 0,
    };
}

/// A glyph as stored in a glyph atlas.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StoredGlyph {
    content_type: Content,
    page: u16,
    frame: u64,
    alloc: Allocation,
    placement_left: i32,
    placement_top: i32,
    size: Size2D<i32, UnknownUnit>,
}
impl StoredGlyph {
    fn create(alloc: &Allocation, placement: &Placement, page: usize, frame: u64, content_type: Content) -> StoredGlyph {
        StoredGlyph {
            content_type,
            page: page as u16,
            frame,
            alloc: alloc.clone(),
            placement_left: placement.left,
            placement_top: placement.top,
            size: placement.size(),
        }
    }
}

/// RGBA color value for text rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorBrush(pub [u8; 4]);
impl Default for ColorBrush {
    fn default() -> Self {
        Self([0, 0, 0, 255])
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Params {
    /// The width of the screen in pixels.
    pub screen_resolution_width: f32,
    /// The height of the screen in pixels.
    pub screen_resolution_height: f32,
    pub srgb: u32,
    pub _pad: u32,
}

pub(crate) struct AtlasPage<ImageType> {
    pub(crate) packer: BucketedAtlasAllocator,
    pub(crate) image: ImageType,
    pub(crate) needs_upload: bool,
}

const SOURCES: &[Source; 3] = &[
    Source::ColorOutline(0),
    Source::ColorBitmap(StrikeWith::BestFit),
    Source::Outline,
];

/// A glyph with the context in which it is being drawn
struct GlyphWithContext {
    glyph: Glyph,
    color: u32,
    font_key: u64,
    font_size: f32,
    quantized_pos_x: i32,
    quantized_pos_y: i32,
    frac_pos_x: f32,
    frac_pos_y: f32,
    subpixel_bin_x: SubpixelBin<4>,
    subpixel_bin_y: SubpixelBin<4>,
}

impl GlyphWithContext {
    fn new(glyph: Glyph, run_x: f32, run_y: f32, font_key: u64, font_size: f32, color: ColorBrush) -> Self {
        let glyph_x = run_x + glyph.x;
        let glyph_y = run_y - glyph.y;

        let (quantized_pos_x, frac_pos_x, subpixel_bin_x) = quantize(glyph_x);
        let (quantized_pos_y, frac_pos_y, subpixel_bin_y) = quantize(glyph_y);

        let color =
          ((color.0[0] as u32) << 24)
        + ((color.0[1] as u32) << 16)
        + ((color.0[2] as u32) << 8)
        + ((color.0[3] as u32) << 0);

        Self { glyph, color, font_key, font_size, quantized_pos_x, quantized_pos_y, frac_pos_x, frac_pos_y, subpixel_bin_x, subpixel_bin_y,}
    }

    fn key(&self) -> GlyphKey {
        GlyphKey {
            font_id: self.font_key,
            glyph_id: self.glyph.id as u16,
            font_size_bits: self.font_size.to_bits(),
            x_bin: self.subpixel_bin_x,
            y_bin: self.subpixel_bin_y,
        }
    }

    fn frac_offset(&self) -> Vector {
        Vector::new(self.frac_pos_x, self.frac_pos_y)
    }
}


trait UselessTrait2 {
    fn size(&self) -> Size2D<i32, UnknownUnit>;
}
impl UselessTrait2 for Placement {
    fn size(&self) -> Size2D<i32, UnknownUnit> {
        size2(self.width as i32, self.height as i32)
    }
}

pub(crate) fn make_decoration_quad(rect: BoundingBox, color: u32, box_index: u32) -> GlyphQuad {
    let x0 = rect.x0 as i32;
    let x1 = rect.x1 as i32;
    let y0 = rect.y0 as i32;
    let y1 = rect.y1 as i32;

    GlyphQuad {
        pos_packed: pack_i32_pair_as_u16(x0, y0),
        dim_packed: pack_u16_pair((x1 - x0) as u32, (y1 - y0) as u32),
        uv_origin_packed: pack_u16_pair(0, 0),
        color,
        flags_and_page: pack_flags_and_page(CONTENT_TYPE_DECORATION, 0),
        box_index,
        _padding1: 0,
        _padding2: 0,
    }
}

impl RenderData {
    /// Create a new RenderData with a specific atlas size.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, atlas_size: u32) -> Self {
        let glyph_cache = LruCache::unbounded_with_hasher(BuildHasherDefault::<FxHasher>::default());

        let mask_atlas_pages = vec![AtlasPage {
            image: GrayImage::from_pixel(atlas_size, atlas_size, Luma([0])),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            needs_upload: true,
        }];

        let color_atlas_pages = vec![AtlasPage {
            image: RgbaImage::from_pixel(atlas_size, atlas_size, Rgba([0, 0, 0, 0])),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            needs_upload: true,
        }];

        let tmp_image = Image::new();

        Self {
            frame: 1,
            tmp_image,
            glyph_cache,
            last_frame_evicted: 0,
            mask_atlas_pages,
            color_atlas_pages,
            glyph_quads: GpuHeap::with_usage(device, 1000, "glyph quads buffer", wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            box_data: GpuSlab::new(device, queue, 30, "box data buffer"),
            group_transforms: {
                let mut slab = GpuSlab::new(device, queue, 16, "group transform buffer");
                let _ = slab.insert(GroupTransform::identity()); // Index 0 is always identity
                slab
            },
            params: Params {
                screen_resolution_width: 0.0,
                screen_resolution_height: 0.0,
                srgb: 0,
                _pad: 0,
            },
            atlas_size,
            needs_texture_array_rebuild: false,
            needs_params_sync: true,
            decoration_quad_handle: None,
            scale_cx: Some(ScaleContext::new()),
            #[cfg(debug_assertions)]
            stats: RenderStats::default(),
        }
    }

    /// Set whether the surface uses sRGB format.
    pub fn set_srgb(&mut self, srgb: bool) {
        let new_srgb = if srgb { 1 } else { 0 };
        if self.params.srgb != new_srgb {
            self.params.srgb = new_srgb;
            self.needs_params_sync = true;
        }
    }

    // for now, we're evicting both masks and colors at the same time even if only one spills over
    // separating them would mean that they can't share the same cache and it would make things more complex
    fn evict_old_glyphs(&mut self) {
        self.last_frame_evicted = self.frame;

        while let Some((_key, value)) = self.glyph_cache.peek_lru() {

            if let Some(stored_glyph) = value {
                if stored_glyph.frame == self.frame {
                    break;
                }

                let page = stored_glyph.page as usize;
                match stored_glyph.content_type {
                    Content::Mask => self.mask_atlas_pages[page].packer.deallocate(stored_glyph.alloc.id),
                    Content::Color => self.color_atlas_pages[page].packer.deallocate(stored_glyph.alloc.id),
                    Content::SubpixelMask => unreachable!()
                }
            }

            self.glyph_cache.pop_lru();
        }

    }

    fn needs_evicting(&self, current_frame: u64) -> bool {
        self.last_frame_evicted != current_frame
    }

    /// Advance the frame counter.
    pub fn advance_frame(&mut self) {
        self.frame += 1;
    }

    /// Update the screen resolution in the render data.
    pub fn update_resolution(&mut self, width: f32, height: f32) {
        if self.params.screen_resolution_width != width || self.params.screen_resolution_height != height {
            self.params.screen_resolution_width = width;
            self.params.screen_resolution_height = height;
            self.needs_params_sync = true;
        }
    }

    /// Prepare a text edit layout for rendering with scrolling and clipping support.
    pub fn prepare_text_edit_layout(&mut self, text_edit: &mut TextEdit, scratch: &mut Vec<GlyphQuad>, encoder: &mut wgpu::CommandEncoder) {
        if text_edit.hidden() {
            return;
        }

        // Update scroll to ensure cursor is visible before rendering
        if text_edit.needs_scroll_update {
            let did_scroll = text_edit.update_scroll_to_cursor();
            if did_scroll {
                text_edit.text_box.shared_mut().scrolled = true;
            }
            text_edit.needs_scroll_update = false;
        }

        let had_relayout = text_edit.text_box.needs_relayout();
        text_edit.refresh_layout();
        #[cfg(debug_assertions)]
        if had_relayout { self.stats.relayouts += 1; }
        if had_relayout { text_edit.text_box.needs_quad_rebuild = true; }

        self.prepare_text_box_layout(&mut text_edit.text_box, scratch, 20, encoder);
    }

    pub(crate) fn prepare_text_box_layout(&mut self, text_box: &mut TextBox, scratch: &mut Vec<GlyphQuad>, spare_capacity: u32, encoder: &mut wgpu::CommandEncoder) {
        let w = self.params.screen_resolution_width;
        let h = self.params.screen_resolution_height;
        let t = text_box.transform.translation;
        let t = if let Some(handle) = text_box.group_transform_index {
            let group = &self.group_transforms[handle.0];
            (t.0 * group.scale + group.offset[0], t.1 * group.scale + group.offset[1])
        } else {
            t
        };
        let offscreen = t.0 < -(5.0 * w) || t.0 > 6.0 * w || t.1 < -(5.0 * h) || t.1 > 6.0 * h;

        if text_box.hidden() || offscreen {
            #[cfg(debug_assertions)] {
                self.stats.boxes_skipped += 1;
            }
            return;
        }
        #[cfg(debug_assertions)] {
            self.stats.boxes_prepared += 1;
        }
        
        let _needs_relayout = text_box.needs_relayout();

        text_box.refresh_layout();
        
        #[cfg(debug_assertions)] {
            if _needs_relayout {
                self.stats.relayouts += 1;            
            }
        }

        if text_box.is_scroll_distance_above_tolerance() {
            text_box.needs_quad_rebuild = true;
        }

        let clip_rect = text_box.effective_clip_rect();
        let screen_clip = text_box.screen_space_clip_rect;
        let scroll_offset = text_box.scroll_offset();

        let box_index = text_box.render_data_info.box_index;
        let group_transform_index = text_box.group_transform_index.map(|h| h.0 as u32).unwrap_or(0);

        // Always update BoxGpu so position/scroll/clip changes are reflected immediately.
        *self.box_data.get_mut(box_index) = create_box_data(
            clip_rect,
            scroll_offset,
            text_box.transform(),
            screen_clip,
            text_box.depth,
            group_transform_index,
            text_box.opacity,
            text_box.color_override,
        );

        if ! text_box.needs_quad_rebuild && text_box.render_data_info.glyph_quad_handle.is_some() {
            // Fast path: quads are still valid, BoxGpu already updated above.
            return;
        }
        let quads = scratch;
        quads.clear();

        // Line culling: clip_rect is already in layout-local coordinates (includes scroll)
        let (clip_top, clip_bottom) = if let Some(clip) = clip_rect {
            (clip.y0 as f32, clip.y1 as f32)
        } else {
            (0.0, self.params.screen_resolution_height)
        };

        for line in text_box.layout.lines() {
            let metrics = line.metrics();

            let line_top = metrics.block_min_coord;
            let line_bottom = metrics.block_max_coord;

            if line_bottom < clip_top - SCROLL_TOLERANCE || line_top > clip_bottom + SCROLL_TOLERANCE {
                continue;
            }

            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(glyph_run) => {
                        self.prepare_glyph_run_into(&glyph_run, box_index as u32, &mut *quads);
                    }
                    PositionedLayoutItem::InlineBox(_inline_box) => {}
                }
            }
        }

        text_box.render_data_info.base_scroll = scroll_offset;

        self.glyph_quads.allocate_or_grow_and_write(&mut text_box.render_data_info.glyph_quad_handle, quads, spare_capacity, encoder);

        text_box.needs_quad_rebuild = false;
    }

    fn prepare_glyph_run_into(
        &mut self,
        glyph_run: &GlyphRun<'_, ColorBrush>,
        box_index: u32,
        buffer: &mut Vec<GlyphQuad>
    ) {
        let mut run_x = glyph_run.offset();
        let run_y = glyph_run.baseline();
        let style = glyph_run.style();

        let run = glyph_run.run();

        let font = run.font();
        let font_size = run.font_size();
        let font_key = font.data.id();

        // partial borrow humiliation ritual
        let mut scale_cx = self.scale_cx.take().unwrap();
        let mut scaler: Option<Scaler> = None;

        for glyph in glyph_run.glyphs() {
            let glyph_ctx = GlyphWithContext::new(glyph, run_x, run_y, font_key, font_size, style.brush);

            if let Some(stored_glyph) = self.glyph_cache.get(&glyph_ctx.key()) {
                if let Some(stored_glyph) = stored_glyph {
                    let quad = make_glyph_quad(&glyph_ctx, stored_glyph, box_index);
                    buffer.push(quad);
                }
            } else {
                // Lazily initialize to skip the cost when all glyphs are cached.
                let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize).unwrap();
                if scaler.is_none() {
                    scaler = Some(
                        scale_cx
                            .builder(font_ref)
                            .size(font_size)
                            .hint(true)
                            .normalized_coords(run.normalized_coords())
                            .build()
                    );
                }
                if let Some((quad, _stored_glyph)) = self.prepare_glyph(&glyph_ctx, scaler.as_mut().unwrap(), box_index) {
                    buffer.push(quad);
                    #[cfg(debug_assertions)] {
                        self.stats.glyph_quads_built += 1;
                    }
                }
            }

            run_x += glyph.advance;
        }

        // put it back
        self.scale_cx = Some(scale_cx);
    }

    fn copy_glyph_to_atlas(&mut self, size: Size2D<i32, UnknownUnit>, alloc: &Allocation, page: usize, content_type: Content) {
        for y in 0..size.height as i32 {
            let src_start = (y as usize) * (size.width as usize);
            let src_slice =
                &self.tmp_image.data[src_start..(src_start + size.width as usize)];

            let dst_y = (alloc.rectangle.min.y + y) as u32;
            let dst_x = alloc.rectangle.min.x as u32;

            match content_type {
                Content::Mask => {
                    let layout = self.mask_atlas_pages[page].image.as_flat_samples().layout;
                    let mut samples = self.mask_atlas_pages[page].image.as_flat_samples_mut();
                    let samples = samples.as_mut_slice();
                    let dst_start =
                    (dst_y as usize) * layout.height_stride + (dst_x as usize) * layout.width_stride;

                samples[dst_start..(dst_start + size.width as usize)].copy_from_slice(src_slice);
                },
                Content::Color => {
                    // todo: rewrite this with a cool copy_from_slice
                    let layout = self.color_atlas_pages[page].image.as_flat_samples().layout;
                    let mut samples = self.color_atlas_pages[page].image.as_flat_samples_mut();
                    let samples = samples.as_mut_slice();

                    // For RGBA, each pixel is 4 bytes
                    for x in 0..size.width as usize {
                        let src_idx = src_start + x;
                        let dst_idx = (dst_y as usize) * layout.height_stride + (dst_x as usize + x) * layout.width_stride;

                        // Copy all 4 channels
                        for c in 0..4 {
                            samples[dst_idx + c] = self.tmp_image.data[src_idx * 4 + c];
                        }
                    }
                },
                Content::SubpixelMask => unreachable!(),
            };
        }

        // Mark the page as dirty since we've modified it
        match content_type {
            Content::Mask => self.mask_atlas_pages[page].needs_upload = true,
            Content::Color => self.color_atlas_pages[page].needs_upload = true,
            Content::SubpixelMask => unreachable!(),
        }
    }

    /// Render a glyph into the `self.tmp_swash_image` buffer
    // this is going to have to return the Content (color/mask) as well
    fn render_glyph(&mut self, glyph: &GlyphWithContext, scaler: &mut Scaler) -> (Content, Placement) {
        self.tmp_image.clear();
        Render::new(SOURCES)
            .format(Format::Alpha)
            .offset(glyph.frac_offset())
            .render_into(scaler, glyph.glyph.id as u16, &mut self.tmp_image);
        return (self.tmp_image.content, self.tmp_image.placement);
    }

    /// Rasterizes the glyph in a texture atlas and returns a Quad that can be used to render it, or None if the glyph was just empty (like a space).
    fn prepare_glyph(&mut self, glyph: &GlyphWithContext, scaler: &mut Scaler, box_index: u32) -> Option<(GlyphQuad, StoredGlyph)> {
        let (content, placement) = self.render_glyph(&glyph, scaler);
        #[cfg(debug_assertions)] {
            self.stats.glyphs_rasterized += 1;
        }
        let size = placement.size();

        // For some glyphs there's no image to store, like spaces.
        if size.is_empty() {
            self.glyph_cache.push(glyph.key(), None);
            return None;
        }

        let n_pages = match content {
            Content::Mask => self.mask_atlas_pages.len(),
            Content::Color => self.color_atlas_pages.len(),
            Content::SubpixelMask => unreachable!(),
        };
        // Try to allocate on existing pages
        for page in 0..n_pages {
            if let Some(alloc) = self.pack_rectangle(size, content, page) {
                return self.store_glyph(glyph, size, &alloc, page, &placement, content, box_index);
            }

            // Try evicting glyphs from previous frames and retry
            if self.needs_evicting(self.frame) {
                self.evict_old_glyphs();

                if let Some(alloc) = self.pack_rectangle(size, content, page) {
                    return self.store_glyph(glyph, size, &alloc, page, &placement, content, box_index);
                }
            }
        }

        // Create a new page and try to allocate there
        let new_page: usize = self.make_new_page(content);
        if let Some(alloc) = self.pack_rectangle(size, content, new_page) {
            return self.store_glyph(glyph, size, &alloc, new_page, &placement, content, box_index);
        }

        // Glyph is too large to fit even in a new empty page. It's time to give up.
        // todo: should probably try to catch these earlier by checking for unreasonable font sizes
        // todo2: technically, we could split the huge glyph across multiple pages, or render it on the surface directly.
        self.glyph_cache.push(glyph.key(), None);
        return None;
    }

    // Helper method to store glyph once allocation is successful
    // todo: don't carry around `size`, alloc probably has the same data
    fn store_glyph(&mut self,
            glyph: &GlyphWithContext,
            size: Size2D<i32, UnknownUnit>,
            alloc: &Allocation,
            page: usize,
            placement: &Placement,
            content_type: Content,
            box_index: u32,
        ) -> Option<(GlyphQuad, StoredGlyph)> {
        self.copy_glyph_to_atlas(size, alloc, page, content_type);
        let stored_glyph = StoredGlyph::create(alloc, placement, page, self.frame, content_type);
        self.glyph_cache.push(glyph.key(), Some(stored_glyph));
        let quad = make_glyph_quad(glyph, &stored_glyph, box_index);
        Some((quad, stored_glyph))
    }

    fn pack_rectangle(&mut self, size: Size2D<i32, UnknownUnit>, content_type: Content, page: usize) -> Option<Allocation> {
        match content_type {
            Content::Mask => self.mask_atlas_pages[page].packer.allocate(size),
            Content::Color => self.color_atlas_pages[page].packer.allocate(size),
            Content::SubpixelMask => unreachable!(),
        }
    }

    fn make_new_page(&mut self, content_type: Content) -> usize {
        let atlas_size = self.atlas_size;

        match content_type {
            Content::Mask => {
                self.mask_atlas_pages.push(AtlasPage {
                    image: GrayImage::from_pixel(atlas_size, atlas_size, Luma([0])),
                    packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
                    needs_upload: true,
                });
                self.needs_texture_array_rebuild = true;
                return self.mask_atlas_pages.len() - 1;
            },
            Content::Color => {
                self.color_atlas_pages.push(AtlasPage {
                    image: RgbaImage::from_pixel(atlas_size, atlas_size, Rgba([0, 0, 0, 0])),
                    packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
                    needs_upload: true,
                });
                self.needs_texture_array_rebuild = true;
                return self.color_atlas_pages.len() - 1;
            },
            Content::SubpixelMask => unreachable!()
        };
    }
}
