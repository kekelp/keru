// Custom shader for demonstrating custom rendering in Keru
// This shader draws an animated radial gradient effect

struct PushConstants {
    // Position and size in pixels
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

var<push_constant> pc: PushConstants;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) pixel_pos: vec2<f32>,
}

@vertex
fn vs_main(@location(0) vertex_pos: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;

    // Scale vertex position by rect size and translate
    let pixel_x = pc.x + vertex_pos.x * pc.width;
    let pixel_y = pc.y + vertex_pos.y * pc.height;

    out.pixel_pos = vec2<f32>(pixel_x, pixel_y);

    // Convert to NDC (normalized device coordinates)
    // We need screen dimensions - for now using a common size
    // In a real app, you'd pass this as a uniform or push constant
    let screen_width = 800.0;
    let screen_height = 600.0;

    let ndc_x = (pixel_x / screen_width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_y / screen_height) * 2.0;  // Flip Y for screen coordinates

    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = vertex_pos;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Create an animated radial gradient effect
    let center = vec2<f32>(0.5, 0.5);
    let to_center = in.uv - center;
    let dist = length(to_center);
    let angle = atan2(to_center.y, to_center.x);

    // Use pixel position to create a time-like value (for animation without passing time)
    let pseudo_time = (in.pixel_pos.x + in.pixel_pos.y) * 0.01;

    // Create swirling colors
    let r = sin(dist * 8.0 + angle * 3.0 + pseudo_time) * 0.5 + 0.5;
    let g = sin(dist * 8.0 + angle * 3.0 + pseudo_time + 2.094) * 0.5 + 0.5;
    let b = sin(dist * 8.0 + angle * 3.0 + pseudo_time + 4.189) * 0.5 + 0.5;

    // Add some vignette effect
    let vignette = smoothstep(0.8, 0.2, dist);

    // Semitransparent with vignette
    let alpha = 0.8 * vignette;

    return vec4<f32>(r, g, b, alpha);
}
