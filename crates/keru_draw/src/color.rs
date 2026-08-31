pub use crate::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Color {
    /// Red component of the color
    pub r: f32,
    /// Green component of the color
    pub g: f32,
    /// Blue component of the color
    pub b: f32,
    /// Alpha component of the color
    pub a: f32,
}

impl Color {
    pub const RED:   Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE:  Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const GREY:  Color = Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
    
    pub const KERU_BLUE:   Color = Color { r: 0.31,g:  0.31, b: 1.0, a: 1.0 };
    pub const KERU_RED:    Color = Color { r: 1.0, g: 0.31, b: 0.31, a: 1.0 };
    pub const KERU_PINK:   Color = Color { r: 0.65, g: 0.31, b: 0.65, a: 1.0 };
    pub const KERU_GREEN:  Color = Color { r: 0.1, g: 0.85, b: 0.1, a: 0.85 };
    pub const KERU_BLACK: Color = Color { r: 0.07, g: 0.07, b: 0.09, a: 1.0 };
    
    pub const DEBUG_RED:   Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 0.3 };
    pub const DEBUG_BLUE:  Color = Color { r: 0.12, g: 0.0, b: 1.0, a: 0.48 };
    
    pub const TRANSPARENT: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.0 };
    pub const GREENSCREEN: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };


    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a packed RGB integer like `0xff00ff` (alpha=1.0).
    pub const fn from_hex(rgb: u32) -> Color {
        Color::rgba_u8(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            255,
        )
    }

    /// Create a color from a hex string like `"#ff00ff"` (alpha=1.0).
    pub const fn from_hex_str(hex: &str) -> Color {
        // Hopefully the const means that this all happens at compile time.
        let b = hex.as_bytes();
        if b.len() != 7 {
            panic!("hex color must be 7 bytes, e.g. \"#ff00ff\"");
        }
        if b[0] != b'#' {
            panic!("hex color must start with '#'");
        }
        Color::rgba_u8(
            hex_bytes_to_u8(b[1], b[2]),
            hex_bytes_to_u8(b[3], b[4]),
            hex_bytes_to_u8(b[5], b[6]),
            255,
        )
    }

    /// Create a color from u8 RGBA values.
    pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Apply alpha to a color.
    pub const fn with_alpha(self, alpha: f32) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    pub fn to_u8_array(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

}

/// The type of gradient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientKind {
    /// Linear gradient from `p0` to `p1`.
    Linear,
    /// Radial gradient centered at `p0`, from `inner_radius` to `outer_radius`.
    Radial,
}

/// A gradient that can be used as a fill, defined in absolute screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    pub color_start: Color,
    pub color_end: Color,
    /// Linear: start point. Radial: center point.
    pub p0: [f32; 2],
    /// Linear: end point. Radial: `[outer_radius, inner_radius]`.
    pub p1: [f32; 2],
    pub kind: GradientKind,
}

impl Gradient {
    /// Linear gradient from absolute point `p0` (color_start) to `p1` (color_end).
    pub fn linear(p0: [f32; 2], p1: [f32; 2], color_start: Color, color_end: Color) -> Self {
        Self { color_start, color_end, p0, p1, kind: GradientKind::Linear }
    }

    /// Radial gradient centered at `center`, transitioning from `inner_radius` (color_start) to `outer_radius` (color_end).
    pub fn radial(center: [f32; 2], outer_radius: f32, inner_radius: f32, color_start: Color, color_end: Color) -> Self {
        Self { color_start, color_end, p0: center, p1: [outer_radius, inner_radius], kind: GradientKind::Radial }
    }
}

/// A handle to a shared gradient that can be created in absolute space and used by multiple shapes.
///
/// Obtained by calling [Canvas::create_gradient()] or [Renderer::create_gradient()];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SharedGradient(pub(crate) u32);

/// Fill style for shapes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorFill {
    Color(Color),
    Gradient(Gradient),
    SharedGradient(SharedGradient),
}

impl Hash for ColorFill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ColorFill::Color(color) => {
                0u8.hash(state);
                color.r.to_bits().hash(state);
                color.g.to_bits().hash(state);
                color.b.to_bits().hash(state);
                color.a.to_bits().hash(state);
            },
            ColorFill::Gradient(g) => {
                1u8.hash(state);
                g.color_start.r.to_bits().hash(state);
                g.color_start.g.to_bits().hash(state);
                g.color_start.b.to_bits().hash(state);
                g.color_start.a.to_bits().hash(state);
                g.color_end.r.to_bits().hash(state);
                g.color_end.g.to_bits().hash(state);
                g.color_end.b.to_bits().hash(state);
                g.color_end.a.to_bits().hash(state);
                g.p0[0].to_bits().hash(state);
                g.p0[1].to_bits().hash(state);
                g.p1[0].to_bits().hash(state);
                g.p1[1].to_bits().hash(state);
                (g.kind as u8).hash(state);
            },
            ColorFill::SharedGradient(handle) => {
                2u8.hash(state);
                handle.0.hash(state);
            },
        }
    }
}

const fn hex_byte_to_u8(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => panic!("invalid hex digit"),
    }
}

const fn hex_bytes_to_u8(high: u8, low: u8) -> u8 {
    hex_byte_to_u8(high) * 16 + hex_byte_to_u8(low)
}