use bytemuck::{Pod, Zeroable};

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

#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod, Hash)]
#[repr(C)]
pub struct VertexColors {
    top_left: Color,
    top_right: Color,
    bottom_left: Color,
    bottom_right: Color,
}
impl VertexColors {
    pub const FLGR_SOVL_GRAD: Self =
        VertexColors::diagonal_gradient_backslash(Color::FLGR_BLUE, Color::FLGR_RED);

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

    // techinically, the blended corners shouldn't be blended with weight 0.5. The weight should depend on the aspect ratio, I think. I don't think that's practical though, and it looks okay like this.
    pub const fn diagonal_gradient_forward_slash(
        bottom_left: Color,
        top_right: Color,
    ) -> VertexColors {
        let blended = bottom_left.blend(top_right, 255 / 2);
        return VertexColors {
            top_left: blended,
            top_right,
            bottom_left,
            bottom_right: blended,
        };
    }

    pub const fn diagonal_gradient_backslash(top_left: Color, bottom_right: Color) -> VertexColors {
        let blended = top_left.blend(bottom_right, 255 / 2);
        return VertexColors {
            top_left,
            top_right: blended,
            bottom_left: blended,
            bottom_right,
        };
    }
}

impl Color {
    pub const FLGR_BLACK: Color = Color {
        r: (0.6 * 255.0) as u8,
        g: (0.3 * 255.0) as u8,
        b: (0.6 * 255.0) as u8,
        a: 255_u8,
    };

    pub const FLGR_DEBUG_RED: Color = Color::rgba(255, 0, 0, 77);

    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const GREEN: Color = Color::rgba(0, 255, 0, 255);
    pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);

    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(255, 255, 255, 0);

    pub const FLGR_BLUE: Color = Color::rgba(26, 26, 255, 255);
    pub const FLGR_RED: Color = Color::rgba(255, 26, 26, 255);
    pub const FLGR_GREEN: Color = Color::rgba(26, 255, 26, 255);

    pub const LIGHT_BLUE: Color = Color {
        r: (0.9 * 255.0) as u8,
        g: (0.7 * 255.0) as u8,
        b: (1.0 * 255.0) as u8,
        a: (0.6 * 255.0) as u8,
    };

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub const fn blend_channel(c1: u8, c2: u8, factor: u8) -> u8 {
        let inv_factor = 255 - factor;
        let res = (c1 as u16 * inv_factor as u16 + c2 as u16 * factor as u16) / 255;
        res as u8
    }

    // todo: in a future version of rust, rewrite with float factor
    // (can't use floats in const functions in current stable rust)
    pub const fn blend(self, other: Color, factor: u8) -> Color {
        Color {
            r: Color::blend_channel(self.r, other.r, factor),
            g: Color::blend_channel(self.g, other.g, factor),
            b: Color::blend_channel(self.b, other.b, factor),
            a: Color::blend_channel(self.a, other.a, factor),
        }
    }
}