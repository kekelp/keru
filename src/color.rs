pub use keru_draw::{ColorFill, GradientType};

pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const GREY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
pub const TRANSPARENT: [f32; 4] = [1.0, 1.0, 1.0, 0.0];

pub const KERU_BLUE: [f32; 4] = [0.31, 0.31, 1.0, 1.0];
pub const KERU_RED: [f32; 4] = [1.0, 0.31, 0.31, 1.0];
pub const KERU_PINK: [f32; 4] = [0.65, 0.31, 0.65, 1.0];
pub const KERU_GREEN: [f32; 4] = [0.1, 1.0, 0.1, 1.0];

pub const DEBUG_RED: [f32; 4] = [1.0, 0.0, 0.0, 0.3];
pub const DEBUG_BLUE: [f32; 4] = [0.12, 0.0, 1.0, 0.48];

/// Create a color from u8 RGBA values.
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
}

/// Apply alpha to a color.
pub const fn with_alpha(color: [f32; 4], alpha: f32) -> [f32; 4] {
    [color[0], color[1], color[2], alpha]
}

pub const KERU_GRAD: ColorFill = ColorFill::Gradient {
    color_start: KERU_BLUE,
    color_end: KERU_RED,
    gradient_type: GradientType::Linear,
    angle: -0.785398, // -45 degrees
};

pub const KERU_GRAD_FW: ColorFill = ColorFill::Gradient {
    color_start: KERU_BLUE,
    color_end: KERU_RED,
    gradient_type: GradientType::Linear,
    angle: 0.785398, // 45 degrees
};

pub const GREENSCREEN: ColorFill = ColorFill::Color([0.0, 1.0, 0.0, 1.0]);
