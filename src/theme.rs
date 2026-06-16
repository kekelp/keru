use crate::*;

#[doc(hidden)]
pub struct Theme {
    // Base colors for light/dark mode
    /// Main background
    pub background: ColorFill2,
    /// Muted background
    pub muted_background: ColorFill2,
    /// Raised elements like cards, buttons
    pub surface: ColorFill2,
    /// Alternative surface for nested elements
    pub surface_alt: ColorFill2,

    // Text colors
    /// Main text
    pub text_primary: ColorFill2,
    /// Less important text
    pub text_secondary: ColorFill2,
    /// Disabled text
    pub text_disabled: ColorFill2,

    // Interactive elements
    /// Main accent color for important actions
    pub primary: ColorFill2,
    /// Hover state for primary
    pub primary_hover: ColorFill2,
    /// Less prominent interactive elements
    pub secondary: ColorFill2,
    /// Hover state for secondary
    pub secondary_hover: ColorFill2,

    // Status colors
    /// Positive actions/states
    pub success: ColorFill2,
    /// Error states
    pub error: ColorFill2,
    /// Warning states
    pub warning: ColorFill2,

    // Common states
    pub disabled: ColorFill2,
    /// Borders and dividers
    pub border: ColorFill2,

    // Optional: Common measurements
    pub border_radius: f32,
    pub border_width: f32,
}

#[doc(hidden)]
pub const KERU_DARK: Theme = Theme {
    muted_background: ColorFill2::Color(Color::new(0.043137256, 0.043137256, 0.05490196, 1.0)),
    background: ColorFill2::Color(Color::new(0.11764706, 0.12156863, 0.16470589, 1.0)),
    surface: ColorFill2::Color(Color::new(0.11764706, 0.12156863, 0.17254902, 1.0)),
    surface_alt: ColorFill2::Color(Color::new(0.14509805, 0.14901961, 0.21176471, 1.0)),

    text_primary: ColorFill2::Color(Color::new(0.8627451, 0.8745098, 0.89411765, 1.0)),
    text_secondary: ColorFill2::Color(Color::new(0.6117647, 0.627451, 0.6901961, 1.0)),
    text_disabled: ColorFill2::Color(Color::new(0.38431373, 0.39215687, 0.45490196, 1.0)),

    primary: ColorFill2::Color(Color::new(0.34901962, 0.6509804, 1.0, 1.0)),
    primary_hover: ColorFill2::Color(Color::new(0.47058824, 0.73333335, 1.0, 1.0)),
    secondary: ColorFill2::Color(Color::new(0.5294118, 0.5411765, 0.7058824, 1.0)),
    secondary_hover: ColorFill2::Color(Color::new(0.6117647, 0.627451, 0.78431374, 1.0)),

    success: ColorFill2::Color(Color::new(0.34117648, 0.7411765, 0.5254902, 1.0)),
    error: ColorFill2::Color(Color::new(0.92156863, 0.34117648, 0.34117648, 1.0)),
    warning: ColorFill2::Color(Color::new(0.9490196, 0.69803923, 0.21960784, 1.0)),

    border: ColorFill2::Color(Color::new(0.1764706, 0.18039216, 0.25882354, 1.0)),
    disabled: ColorFill2::Color(Color::new(0.19215687, 0.19607843, 0.26666668, 1.0)),

    border_radius: 4.0,
    border_width: 1.0,
};