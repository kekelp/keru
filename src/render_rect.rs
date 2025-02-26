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

    pub vertex_colors: VertexColors, 
    
    pub z: f32,                     // (f32) depth information
    pub last_hover: f32,            // (f32) hover timestamp
    pub last_click: f32,            // (f32) click timestamp
    pub shape_data: f32,                // (f32) radius
    
    pub flags: u32,                 // (u32) bitfield flags
    pub _padding: u32,        // (u32) next free block index

    pub clip_rect: XyRect,          // (f32, f32) for each corner

    // this is currently used for click resolution, but it's not used for anything on the gpu.
    // in the future, I would like to have a separate structure for click resolution, and remove the Id from this structure.
    pub id: Id,
}

impl RenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 16] {
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
            11 => Float32, // shape_data (rect radius/ring width)

            12 => Uint32, // flags
            13 => Uint32, // slab_next_free

            // rect (XyRect): 2 x Float32x2
            14 => Float32x2, // rect.x_min, rect.y_min
            15 => Float32x2, // rect.x_max, rect.y_max
            
            // 16 => Uint32x2, // id. it's actually a u64, but it doesn't look like wgpu understands u64s.
        ]
    }


    pub fn read_shape(&self) -> Shape {
        // Extract the first 8 bits (bits 0–7) from `flags` to determine the `RenderShape`
        let shape_byte = (self.flags & 0x000000FF) as u8; // Mask out all but the least significant 8 bits
        let render_shape = match shape_byte {
            0 => RenderShape::Rectangle,
            1 => RenderShape::Circle,
            2 => RenderShape::Ring,
            _ => panic!("Invalid shape byte: {}", shape_byte),
        };


        return match render_shape {
            RenderShape::Rectangle => Shape::Rectangle {
                corner_radius: self.shape_data,
            },
            RenderShape::Circle => Shape::Circle,
            RenderShape::Ring => Shape::Ring {
                width: self.shape_data,
            },
        };
    }
}

#[rustfmt::skip]
impl RenderRect {
    // the first 8 bits are for RenderShape
    
    pub const CLICK_ANIMATION: u32 = 1 << 8;  // 0b00000000_00000000_00000001_00000000
    pub const OUTLINE_ONLY:    u32 = 1 << 9;  // 0b00000000_00000000_00000010_00000000 
    pub const HOVERED:         u32 = 1 << 10; // 0b00000000_00000000_00000100_00000000 

    // bits 11, 12, 13, 14 are for rounded corners

    pub const EMPTY_FLAGS: u32 = 0;
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub(crate) enum RenderShape {
    Rectangle = 0,
    Circle = 1,
    Ring = 2,
}

const SHAPE_BITS_MASK: u32 = 0b11111111_11111111_11111111_00000000;
impl RenderShape {
    fn write_into_least_significant_8_bits(flags: u32, shape: u8) -> u32 {
        // Clear the first 8 bits (bits 0–7)
        let cleared = flags & SHAPE_BITS_MASK;
        // Insert the u8 value into the first 8 bits
        return cleared | (shape as u32);
    }
}

const ROUNDED_CORNERS_MASK: u32 = !(0b00001111 << 11);

impl RenderShape {
    fn write_corners(flags: &mut u32, corners: RoundedCorners) {
        let cleared = *flags & ROUNDED_CORNERS_MASK;
        *flags = cleared | ((corners.bits() as u32) << 11);
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
        if self.hovered {
            flags |= RenderRect::HOVERED;
        }


        flags = RenderShape::write_into_least_significant_8_bits(flags, self.params.rect.shape.render_shape() as u8);
        
        RenderShape::write_corners(&mut flags, self.params.rect.rounded_corners);

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
            clip_rect: self.clip_rect.to_graphics_space(),
            vertex_colors: self.params.rect.vertex_colors,
            last_hover: self.hover_timestamp,
            last_click: self.last_click,
            id: self.id,
            z: self.z,
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
    //         // in inspect mode, draw invisible rects as well.
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