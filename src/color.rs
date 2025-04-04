use bytemuck::{Pod, Zeroable};

/// A `rgba` color
#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod, Hash)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const fn alpha(mut self, alpha: u8) -> Self {
        self.a = alpha;
        return self;
    }
}

/// A node's vertex colors.
#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod, Hash)]
#[repr(C)]
pub struct VertexColors {
    top_left: Color,
    top_right: Color,
    bottom_left: Color,
    bottom_right: Color,
}
impl VertexColors {
    pub const GREENSCREEN: Self = VertexColors::flat(Color::rgba_f(0.0, 1.0, 0.0, 0.7));

    pub const KERU_GRAD: Self =
        VertexColors::diagonal_gradient_backslash(Color::KERU_BLUE, Color::KERU_RED);

    pub const KERU_GRAD_FW: Self =
        VertexColors::diagonal_gradient_forward_slash(Color::KERU_BLUE, Color::KERU_RED);


    pub const TEST: Self = Self {
        top_left: Color::rgba(255, 0, 0, 255),
        top_right: Color::rgba(0, 255, 0, 255),
        bottom_left: Color::rgba(0, 0, 255, 255),
        bottom_right: Color::rgba(255, 255, 255, 255),
    };
    pub const TEST2: Self = Self {
        top_left: Color::WHITE,
        top_right: Color::RED,
        bottom_left: Color::WHITE,
        bottom_right: Color::WHITE,
    };
    pub const fn new(tl: Color, tr: Color, bl: Color, br: Color) -> VertexColors {
        return VertexColors {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br,
        };
    }

    pub const fn flat(color: Color) -> VertexColors {
        return VertexColors::new(color, color, color, color);
    }

    pub const fn h_gradient(left: Color, right: Color) -> VertexColors {
        return VertexColors::new(left, right, left, right);
    }

    pub const fn v_gradient(top: Color, bottom: Color) -> VertexColors {
        return VertexColors::new(top, top, bottom, bottom);
    }

    // technically, the blended corners shouldn't be blended with weight 0.5. The weight should depend on the aspect ratio, I think. I don't think that's practical though, and it looks okay like this.
    pub const fn diagonal_gradient_forward_slash(
        bottom_left: Color,
        top_right: Color,
    ) -> VertexColors {
        let blended = bottom_left.blend(top_right, 0.5);
        return VertexColors {
            top_left: blended,
            top_right,
            bottom_left,
            bottom_right: blended,
        };
    }

    pub const fn diagonal_gradient_backslash(top_left: Color, bottom_right: Color) -> VertexColors {
        let blended = top_left.blend(bottom_right, 0.5);
        return VertexColors {
            top_left,
            top_right: blended,
            bottom_left: blended,
            bottom_right,
        };
    }
}

impl Color {
    pub const KERU_BLACK: Color = Color {
        r: (0.07 * 255.0) as u8,
        g: (0.07 * 255.0) as u8,
        b: (0.09 * 255.0) as u8,
        a: 255_u8,
    };

    pub const KERU_DEBUG_RED: Color = Color::rgba(255, 0, 0, 77);

    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const GREEN: Color = Color::rgba(0, 255, 0, 255);
    pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);
    
    pub const GREY: Color = Color::rgba(50, 50, 50, 255);
    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(255, 255, 255, 0);

    pub const KERU_BLUE: Color = Color::rgba(26, 26, 255, 255);
    pub const KERU_RED: Color = Color::rgba(255, 26, 26, 255);
    pub const KERU_PINK: Color = Color::rgba(141, 52, 141, 180);
    pub const KERU_GREEN: Color = Color::rgba(26, 255, 26, 255);

    pub const LIGHT_BLUE: Color = Color {
        r: (0.9 * 255.0) as u8,
        g: (0.7 * 255.0) as u8,
        b: (1.0 * 255.0) as u8,
        a: (0.6 * 255.0) as u8,
    };

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub const fn rgba_f(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: (r * 255.0) as u8, g: (g * 255.0) as u8, b: (b * 255.0) as u8, a: (a * 255.0) as u8 }
    }

    pub const fn blend_channel(c1: u8, c2: u8, factor: f32) -> u8 {
        // Ensure factor is clamped between 0.0 and 1.0
        let clamped_factor = if factor < 0.0 {
            0.0
        } else if factor > 1.0 {
            1.0
        } else {
            factor
        };

        let res = (c1 as f32 * (1.0 - clamped_factor)) + (c2 as f32 * clamped_factor);
        // manual round (f32::round not const yet apparently)
        // todo: check this in future rust
        return (res + 0.5) as u8;
    }

    pub const fn blend(self, other: Color, factor: f32) -> Color {
        Color {
            r: Color::blend_channel(self.r, other.r, factor),
            g: Color::blend_channel(self.g, other.g, factor),
            b: Color::blend_channel(self.b, other.b, factor),
            a: Color::blend_channel(self.a, other.a, factor),
        }
    }
}