use crate::*;

#[doc(hidden)]
pub struct Theme {
    // Base colors for light/dark mode
    /// Main background
    pub background: ColorFill,
    /// Muted background
    pub muted_background: ColorFill,
    /// Raised elements like cards, buttons
    pub surface: ColorFill,
    /// Alternative surface for nested elements
    pub surface_alt: ColorFill,

    // Text colors
    /// Main text
    pub text_primary: ColorFill,
    /// Less important text
    pub text_secondary: ColorFill,
    /// Disabled text
    pub text_disabled: ColorFill,

    // Interactive elements
    /// Main accent color for important actions
    pub primary: ColorFill,
    /// Hover state for primary
    pub primary_hover: ColorFill,
    /// Less prominent interactive elements
    pub secondary: ColorFill,
    /// Hover state for secondary
    pub secondary_hover: ColorFill,

    // Status colors
    /// Positive actions/states
    pub success: ColorFill,
    /// Error states
    pub error: ColorFill,
    /// Warning states
    pub warning: ColorFill,

    // Common states
    pub disabled: ColorFill,
    /// Borders and dividers
    pub border: ColorFill,

    // Optional: Common measurements
    pub border_radius: f32,
    pub border_width: f32,
}

#[doc(hidden)]
pub const KERU_DARK: Theme = Theme {
    muted_background: ColorFill::Color(rgba(11, 11, 14, 255)),
    background: ColorFill::Color(rgba(30, 31, 42, 255)),
    surface: ColorFill::Color(rgba(30, 31, 44, 255)),
    surface_alt: ColorFill::Color(rgba(37, 38, 54, 255)),

    text_primary: ColorFill::Color(rgba(220, 223, 228, 255)),
    text_secondary: ColorFill::Color(rgba(156, 160, 176, 255)),
    text_disabled: ColorFill::Color(rgba(98, 100, 116, 255)),

    primary: ColorFill::Color(rgba(89, 166, 255, 255)),
    primary_hover: ColorFill::Color(rgba(120, 187, 255, 255)),
    secondary: ColorFill::Color(rgba(135, 138, 180, 255)),
    secondary_hover: ColorFill::Color(rgba(156, 160, 200, 255)),

    success: ColorFill::Color(rgba(87, 189, 134, 255)),
    error: ColorFill::Color(rgba(235, 87, 87, 255)),
    warning: ColorFill::Color(rgba(242, 178, 56, 255)),

    border: ColorFill::Color(rgba(45, 46, 66, 255)),
    disabled: ColorFill::Color(rgba(49, 50, 68, 255)),

    border_radius: 4.0,
    border_width: 1.0,
};
