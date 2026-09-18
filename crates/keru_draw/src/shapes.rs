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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub(crate) struct TextureGpu {
    pub uv_origin: [f32; 2],   // atlas pixel coords (top-left of the image in the atlas)
    pub uv_size: [f32; 2],     // atlas pixel size of the image
    pub page: u32,             // atlas layer
    pub flags: u32,            // bit 0=nine-slice/tiling enabled; bits 1-2=tile_x; bits 3-4=tile_y; bit 5=absolute space
    pub nine_slice: [f32; 4],  // insets: left, right, top, bottom
    pub abs_origin: [f32; 2],  // absolute-space anchor rect origin (draw-call coords)
    pub abs_size: [f32; 2],    // absolute-space anchor rect size
    pub _pad: [f32; 2],        // to 16 floats (one ResourceSlot)
}

pub(crate) const TEXTURE_ABSOLUTE_BIT: u32 = 1 << 5;

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
    pub corner_radius: f32,
    pub border_thickness: f32,
    pub gradient_direction: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub rounded_corners: u32, // bitflags: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right
    pub blur_radius: f32,
    pub texture_index: u32,
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
    pub dash_length: f32,            // 0 = no dashing, >0 = dash length in pixels
    pub dash_offset: f32,            // offset for dash pattern alignment
    pub blur_radius: f32,
    pub texture_index: u32,
    pub pie_stroke_thickness: f32, // pie only: 0=filled, >0=hollow outline
    pub pie_corner_radius: f32,    // pie only: rounds center point and arc-edge corners
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
    pub blur_radius: f32,
    pub texture_index: u32,
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
    pub blur_radius: f32,
    pub texture_index: u32,
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
    pub blur_radius: f32,
    pub stroke_thickness: f32,       // 0 = filled, >0 = stroke only
    pub texture_index: u32,
    pub corner_radius: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HexagonGpu {
    pub center: [f32; 2],
    pub size: f32,
    pub rotation: f32,
    pub gradient_direction: [f32; 2],
    pub stroke_thickness: f32,
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub blur_radius: f32,
    pub texture_index: u32,
    pub corner_radius: f32,
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
pub struct PolygonGpu {
    pub bbox_min: [f32; 2],
    pub bbox_max: [f32; 2],
    pub gradient_index: u32,
    pub vert_offset: u32,
    pub vert_count: u32,
    pub stroke_thickness: f32,  // 0 = filled, >0 = stroke only
    pub blur_radius: f32,
    pub texture_index: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PrimitiveGpu(pub [f32; 32]);

pub struct Shapes {
    pub(crate) gradient_indices: Vec<usize>,
    pub(crate) primitives: GpuVec<PrimitiveGpu>,
    pub(crate) polygon_vertices: GpuVec<[f32; 2]>,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let primitives = GpuVec::new(device, 64, "keru_draw primitives");
        let polygon_vertices = GpuVec::new(device, 64, "keru_draw polygon vertices");

        Self {
            gradient_indices: Vec::new(),
            primitives,
            polygon_vertices,
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
        self.polygon_vertices.clear();
    }

    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let primitives_realloc = self.primitives.load_to_gpu(device, queue);
        let vertices_realloc = self.polygon_vertices.load_to_gpu(device, queue);
        primitives_realloc || vertices_realloc
    }
}
