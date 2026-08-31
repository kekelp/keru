use crate::gpu_vec::GpuVec;
use crate::{Color, Gradient, GradientKind};


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub(crate) struct GradientGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub p0: [f32; 2], // gradient start point (absolute screen coords); for radial: center
    pub p1: [f32; 2], // gradient end point; for radial: [outer_radius, inner_radius]
    pub gradient_type: u32, // 0=solid, 1=linear, 2=radial
    pub _pad: [u32; 3],
}

impl GradientGpu {
    pub fn solid(color: Color) -> Self {
        Self { color_start: color, color_end: color, p0: [0.0; 2], p1: [0.0; 2], gradient_type: 0, _pad: [0; 3] }
    }
}

impl Gradient {
    pub(crate) fn to_gpu(self) -> GradientGpu {
        let gradient_type = match self.kind {
            GradientKind::Linear => 1,
            GradientKind::Radial => 2,
        };
        GradientGpu {
            color_start: self.color_start,
            color_end: self.color_end,
            p0: self.p0,
            p1: self.p1,
            gradient_type,
            _pad: [0; 3],
        }
    }
}










#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct RectangleGpu {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub corner_radius: f32,
    pub border_thickness: f32,
    pub gradient_direction: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub rounded_corners: u32, // bitflags: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub blur_radius: f32,
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: [f32; 3],           // padding to 128 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub center: [f32; 2],
    pub radii: [f32; 2],      // [inner_radius, outer_radius]
    pub angles: [f32; 2],     // [start_angle, end_angle] in radians
    pub gradient_direction: [f32; 2],
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub dash_length: f32,            // 0 = no dashing, >0 = dash length in pixels
    pub dash_offset: f32,            // offset for dash pattern alignment
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: [f32; 2],           // padding to 128 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SegmentGpu {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub thickness_dash: [f32; 4], // [thickness, dash_length, dash_offset, stroke_thickness]
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub lattice_size: [f32; 2],
    pub gradient_direction: [f32; 2],
    pub line_thickness: f32,
    pub gradient_index: u32, // index into gradients buffer
    pub grid_type: u32, // 0=square, 1=hex
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleGpu {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub gradient_direction: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub stroke_thickness: f32,       // 0 = filled, >0 = stroke only
    pub _tri_pad: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HexagonGpu {
    pub center: [f32; 2],
    pub size: f32,
    pub rotation: f32,
    pub gradient_direction: [f32; 2],
    pub stroke_thickness: f32,
    pub texture_page: u32,
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub texture_uv_origin: [f32; 2],
    pub texture_uv_size: [f32; 2],
    pub blur_radius: f32,
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: f32,                // padding to 112 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadraticBezierGpu {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub thickness: f32,
    pub blur_radius: f32,
    pub gradient_index: u32, // index into gradients buffer (replaces color: Color, same 16 bytes)
    pub stroke_thickness: f32,  // 0 = filled, >0 = stroke only
    pub _cu_pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PrimitiveGpu(pub [f32; 32]);

pub struct Shapes {
    pub(crate) gradient_indices: Vec<usize>,
    pub(crate) primitives: GpuVec<PrimitiveGpu>,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let primitives = GpuVec::new(device, 64, "keru_draw primitives");

        Self {
            gradient_indices: Vec::new(),
            primitives,
        }
    }

    pub(crate) fn push_primitive<T: bytemuck::Pod>(&mut self, primitive: T) -> u32 {
        let src = bytemuck::bytes_of(&primitive);
        debug_assert!(src.len() <= 128, "primitive is larger than the 128-byte PrimitiveSlot");
        let mut slot = PrimitiveGpu([0.0; 32]);
        bytemuck::bytes_of_mut(&mut slot)[..src.len()].copy_from_slice(src);
        let index = self.primitives.len() as u32;
        self.primitives.push(slot);
        index
    }

    pub fn clear(&mut self) {
        // gradient_indices are drained by Renderer::clear_for_new_frame before calling this
        self.primitives.clear();
    }

    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        self.primitives.load_to_gpu(device, queue)
    }
}
