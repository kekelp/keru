use bytemuck::{Pod, Zeroable};
use node::Node;
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
    pub shape_data: f32,                // (f32) radius
    
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

    // the last 4 bits are for RenderShape

    pub const EMPTY_FLAGS: u32 = 0;
}

#[derive(Copy, Clone)]
pub(crate) enum RenderShape {
    Rectangle = 0,
    Circle = 1,
    Ring = 2,
}

impl RenderShape {
    fn write_into_last_8_bits(&self, value: &mut u32) {
        // Clear the last 8 bits and insert the Shape value
        *value = (*value & !0xFF) | (*self as u8 as u32);
    }
}

impl Shape {
    pub(crate) fn render_shape(&self) -> RenderShape {
        match self {
            Shape::Rectangle { .. } => RenderShape::Rectangle,
            Shape::Circle => RenderShape::Circle,
            Shape::Ring { .. } => RenderShape::Ring,
        }
    }

    pub fn shape_data(&self) -> f32 {
        match *self {
            Shape::Rectangle { corner_radius } => corner_radius,
            Shape::Circle => 0.0,
            Shape::Ring { width } => width,
        }
    } 
}


impl Node {
    pub(crate) fn render_rect(&self, draw_even_if_invisible: bool, image_texcoords: Option<Xy<[f32; 2]>>) -> Option<RenderRect> {
        if ! draw_even_if_invisible && ! self.params.rect.visible {
            return None;
        }

        let mut flags = RenderRect::EMPTY_FLAGS;
        if self.params.interact.click_animation {
            flags |= RenderRect::CLICK_ANIMATION;
        }
        if self.params.rect.outline_only {
            flags |= RenderRect::OUTLINE_ONLY;
        }

        self.params.rect.shape.render_shape().write_into_last_8_bits(&mut flags);

        let tex_coords = if let Some(image_texcoords) = image_texcoords {
            image_texcoords
        } else {
            // magic coords
            // todo: demagic
            Xy {
                x: [0.9375, 0.9394531],
                y: [0.00390625 / 2.0, 0.0],
            }
        };

        return Some(RenderRect {
            rect: self.rect.to_graphics_space(),
            vertex_colors: self.params.rect.vertex_colors,
            last_hover: self.last_hover,
            last_click: self.last_click,
            id: self.id,
            z: 0.0,
            shape_data: self.params.rect.shape.shape_data(),
            tex_coords,
            flags,
            _padding: 0,
        })
    }

    // pub(crate) fn image_rect(&self) -> Option<RenderRect> {
    //     let mut image_flags = RenderRect::EMPTY_FLAGS;
    //     if self.params.interact.click_animation {
    //         image_flags |= RenderRect::CLICK_ANIMATION;
    //     }

    //     if let Some(image) = self.imageref {
    //         // in debug mode, draw invisible rects as well.
    //         // usually these have filled = false (just the outline), but this is not enforced.

    //         return Some(RenderRect {
    //             rect: self.rect.to_graphics_space(),
    //             vertex_colors: self.params.rect.vertex_colors,
    //             last_hover: self.last_hover,
    //             last_click: self.last_click,
    //             id: self.id,
    //             z: 0.0,
    //             shape_data: BASE_RADIUS,

    //             tex_coords: image.tex_coords,
    //             flags: image_flags,
    //             _padding: 0,
    //         });
    //     }

    //     return None;
    // }
}