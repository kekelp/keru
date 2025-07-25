use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, VertexAttribute};

// todo: don't do this
pub(crate) const DUMB_MAGIC_TEX_COORDS: XyRect = Xy {
    x: [0.9375, 0.9394531],
    y: [0.00390625 / 2.0, 0.0],
};

use crate::*;

/// A struct with the information needed to render an ui rectangle on the screen.
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

impl Ui {
    pub(crate) fn render_rect_i(&self, i: NodeI, draw_even_if_invisible: bool, image_texcoords: Option<Xy<[f32; 2]>>, without_padding: bool) -> Option<RenderRect> {
        let node = &self.nodes[i];
        if ! draw_even_if_invisible && ! node.params.rect.visible {
            return None;
        }

        let outline_only = if image_texcoords.is_some() {
            false
        } else {
            node.params.rect.outline_only
        };

        let mut flags = RenderRect::EMPTY_FLAGS;
        if node.params.interact.click_animation {
            flags |= RenderRect::CLICK_ANIMATION;
        }
        if outline_only {
            flags |= RenderRect::OUTLINE_ONLY;
        }
        if node.hovered {
            flags |= RenderRect::HOVERED;
        }

        flags = RenderShape::write_into_least_significant_8_bits(flags, node.params.rect.shape.render_shape() as u8);
        
        RenderShape::write_corners(&mut flags, node.params.rect.rounded_corners);

        let tex_coords = if let Some(image_texcoords) = image_texcoords {
            image_texcoords
        } else {
            DUMB_MAGIC_TEX_COORDS
        };

        let size = self.sys.unifs.size;

        let rect = if without_padding {
            dbg!(self.nodes[i].debug_name());
            let padding = self.pixels_to_frac2(node.params.layout.padding);
            let mut rect_without_padding = node.rect;
            rect_without_padding.x[0] += padding.x;
            rect_without_padding.x[1] -= padding.x;
            rect_without_padding.y[0] += padding.y;
            rect_without_padding.y[1] -= padding.y;
            rect_without_padding.to_graphics_space_rounded(size)
        } else {
            node.rect.to_graphics_space_rounded(size)
        };

        return Some(RenderRect {
            rect: rect,
            clip_rect: node.clip_rect.to_graphics_space_rounded(size),
            vertex_colors: node.params.rect.vertex_colors,
            last_hover: node.hover_timestamp,
            last_click: node.last_click,
            z: node.z,
            shape_data: node.params.rect.shape.shape_data(),
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