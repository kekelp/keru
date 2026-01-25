struct PushConstants {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    time: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

var<push_constant> data: PushConstants;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate quad vertices from vertex index (triangle strip: 0,1,2,3)
    // 0: bottom-left, 1: bottom-right, 2: top-left, 3: top-right
    let u = f32(vertex_index & 1u);
    let v = f32((vertex_index >> 1u) & 1u);

    out.uv = vec2<f32>(u, v);

    // Interpolate between min and max using normalized coords
    let x = mix(data.min_x, data.max_x, u);
    let y = mix(data.min_y, data.max_y, v);

    // Convert from normalized (0-1) to NDC (-1 to 1)
    let ndc_x = x * 2.0 - 1.0;
    let ndc_y = 1.0 - y * 2.0;  // Flip Y for screen coordinates

    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Create an animated radial gradient effect
    let center = vec2<f32>(0.5, 0.5);
    let to_center = in.uv - center;
    let dist = length(to_center);
    let angle = atan2(to_center.y, to_center.x);

    // Create swirling colors using time from push constants
    let r = sin(dist * 8.0 + angle * 3.0 + data.time) * 0.5 + 0.5;
    let g = sin(dist * 8.0 + angle * 3.0 + data.time + 2.094) * 0.5 + 0.5;
    let b = sin(dist * 8.0 + angle * 3.0 + data.time + 4.189) * 0.5 + 0.5;

    // Add some vignette effect
    let vignette = smoothstep(0.8, 0.2, dist);

    // Semitransparent with vignette
    let alpha = 0.7 * vignette;

    return vec4<f32>(r, g, b, alpha);
}
