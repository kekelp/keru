use crate::*;

#[doc(hidden)]
pub struct Theme {
    // Base colors for light/dark mode
    /// Main background
    pub background: VertexColors,          
    /// Muted background
    pub muted_background: VertexColors,
    /// Raised elements like cards, buttons
    pub surface: VertexColors,             
    /// Alternative surface for nested elements
    pub surface_alt: VertexColors,         
    
    
    // Text colors
    /// Main text
    pub text_primary: VertexColors,        
    /// Less important text
    pub text_secondary: VertexColors,      
    /// Disabled text
    pub text_disabled: VertexColors,       
    
    
    // Interactive elements
    /// Main accent color for important actions
    pub primary: VertexColors,             
    /// Hover state for primary
    pub primary_hover: VertexColors,       
    /// Less prominent interactive elements
    pub secondary: VertexColors,           
    /// Hover state for secondary
    pub secondary_hover: VertexColors,     
    
    
    // Status colors
    /// Positive actions/states
    pub success: VertexColors,             
    /// Error states
    pub error: VertexColors,              
    /// Warning states
    pub warning: VertexColors,            
    
    
    // Common states
    pub disabled: VertexColors,           
    /// Disabled elements
    pub border: VertexColors,             
    /// Borders and dividers
    
    // Optional: Common measurements
    pub border_radius: f32,        // Default corner rounding
    pub border_width: f32,         // Default border thickness
}

#[doc(hidden)]
pub const KERU_DARK: Theme = Theme {
    // Dark base colors
    muted_background: VertexColors::flat(Color { r: 11, g: 11, b: 14, a: 255 }),      // Muted background
    background: VertexColors::flat(Color { r: 26, g: 27, b: 38, a: 255 }),    // Very dark blue-tinted grey
    surface: VertexColors::flat(Color { r: 30, g: 31, b: 44, a: 255 }),       // Slightly lighter
    surface_alt: VertexColors::flat(Color { r: 37, g: 38, b: 54, a: 255 }),   // For contrast against surface

    // Text VertexColors::flat(colors
    text_primary: VertexColors::flat(Color { r: 220, g: 223, b: 228, a: 255 }),   // Off-white
    text_secondary: VertexColors::flat(Color { r: 156, g: 160, b: 176, a: 255 }), // Medium grey
    text_disabled: VertexColors::flat(Color { r: 98, g: 100, b: 116, a: 255 }),   // Darker grey

    // Interactive elements - light blue scheme
    primary: VertexColors::flat(Color { r: 89, g: 166, b: 255, a: 255 }),     // Bright light blue
    primary_hover: VertexColors::flat(Color { r: 120, g: 187, b: 255, a: 255 }), // Lighter blue
    secondary: VertexColors::flat(Color { r: 135, g: 138, b: 180, a: 255 }),     // Muted purple-blue
    secondary_hover: VertexColors::flat(Color { r: 156, g: 160, b: 200, a: 255 }), // Slightly brighter purple-blue

    // Status indicators
    success: VertexColors::flat(Color { r: 87, g: 189, b: 134, a: 255 }),     // Green
    error: VertexColors::flat(Color { r: 235, g: 87, b: 87, a: 255 }),        // Red
    warning: VertexColors::flat(Color { r: 242, g: 178, b: 56, a: 255 }),     // Orange

    // UI elements
    border: VertexColors::flat(Color { r: 45, g: 46, b: 66, a: 255 }),        // Subtle border
    disabled: VertexColors::flat(Color { r: 49, g: 50, b: 68, a: 255 }),      // Muted background

    border_radius: 4.0,
    border_width: 1.0,
};
