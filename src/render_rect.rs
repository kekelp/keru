use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, VertexAttribute};

use crate::*;

/// A struct with the information needed to render an ui rectangle on the screen.
/// Despite the name, it is also used for checking for click resolution.
/// The Ui state keeps a Vec of these.
#[repr(C)]
#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
pub(crate) struct RenderRect {
    pub rect: XyRect,               // (f32, f32) for each corner
    pub tex_coords: XyRect,         // (f32, f32) for texture coordinates
    pub vertex_colors: VertexColors, // (u8, u8, u8, u8) colors
    
    pub z: f32,                     // (f32) depth information
    pub last_hover: f32,            // (f32) hover timestamp
    pub last_click: f32,            // (f32) click timestamp
    pub radius: f32,                // (f32) radius
    
    pub flags: u32,                 // (u32) bitfield flags
    pub _padding: u32,        // (u32) next free block index

    // this is currently used for click resolution, but it's not used for anything on the gpu.
    // in the future, I would like to have a separate structure for click resolution, and remove the Id from this structure.
    pub id: Id,
}

impl RenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 15] {
        vertex_attr_array![
            // rect (XyRect): 2 x Float32x2
            0 => Float32x2, // rect.x_min, rect.y_min
            1 => Float32x2, // rect.x_max, rect.y_max

            // tex_coords (XyRect): 2 x Float32x2
            2 => Float32x2, // tex_coords.x_min, tex_coords.y_min
            3 => Float32x2, // tex_coords.x_max, tex_coords.y_max

            // vertex_colors (VertexColors): 4 x Uint8x4
            4 => Uint8x4, // vertex_colors[0]
            5 => Uint8x4, // vertex_colors[1]
            6 => Uint8x4, // vertex_colors[2]
            7 => Uint8x4, // vertex_colors[3]

            8 => Float32,  // z
            9 => Float32,  // last_hover
            10 => Float32, // last_click
            11 => Float32, // radius

            12 => Uint32, // flags
            13 => Uint32, // slab_next_free
            
            14 => Uint32x2, // id. it's actually a u64, but it doesn't look like wgpu understands u64s.
        ]
    }
}

#[rustfmt::skip]
impl RenderRect {
    pub const CLICK_ANIMATION: u32 = 1 << 0;
    pub const OUTLINE_ONLY:    u32 = 1 << 1;

    pub const EMPTY_FLAGS: u32 = 0;
}