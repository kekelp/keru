// Color picker shader, driven entirely by push constants (no vertex buffer, no bind groups).
//
// It draws one quad (a triangle strip of 4 vertices) covering the node's rect, given in Keru
// "graphics space" (0..1, top-left origin). `mode` selects what to draw:
//   0.0 -> OkLab hue wheel (a real 60px-wide ring; hue = angle around the center)
//   1.0 -> OkLab lightness/chroma square (x = lightness, y = chroma)
//
// The fragment logic (gamut-edge alpha feathering, the antialiased pixel-accurate ring, and the
// selected-hue marker) is ported from the original shader; only the data delivery changed.

const PI: f32 = 3.141592653589793;

// All scalars so the block packs tightly to match the Rust `PushConstants` (10 x f32).
struct PushConstants {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    // hue in radians, chroma, lightness
    hue: f32,
    chroma: f32,
    lightness: f32,
    win_x: f32,
    win_y: f32,
    mode: f32,
}

var<push_constant> data: PushConstants;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // uv in [-1, 1], y pointing down (screen orientation).
    @location(0) uv: vec2<f32>,
    // uv expressed in pixels relative to the rect center.
    @location(1) pixel_uv: vec2<f32>,
    // half the rect size, in pixels.
    @location(2) half_size: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let i_x = f32(vertex_index & 1u);
    let i_y = f32((vertex_index >> 1u) & 1u);

    let x = mix(data.min_x, data.max_x, i_x);
    let y = mix(data.min_y, data.max_y, i_y);

    let ndc_x = x * 2.0 - 1.0;
    let ndc_y = 1.0 - y * 2.0;

    // [-1, 1], y down (i_y = 0 is the top of the rect).
    let uv = vec2<f32>(i_x * 2.0 - 1.0, i_y * 2.0 - 1.0);

    let half_size = vec2<f32>(
        (data.max_x - data.min_x) * data.win_x * 0.5,
        (data.max_y - data.min_y) * data.win_y * 0.5,
    );
    let pixel_uv = uv * half_size;

    return VertexOutput(vec4<f32>(ndc_x, ndc_y, 0.0, 1.0), uv, pixel_uv, half_size);
}

// Convert OkLCH (hue normalized to [0, 1]) to linear sRGB, with alpha for gamut-edge feathering.
fn hcl_rgb_with_alpha(hcl: vec3<f32>) -> vec4<f32> {
    let h = hcl.x * 2.0 * PI;
    let c = hcl.y;
    let l = hcl.z;

    // HCL -> Lab
    let lab = vec3f(l, c * cos(h), c * sin(h));

    // Lab -> LMS
    var lms = vec3f(
        lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
        lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
        lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z,
    );

    lms.x = pow(lms.x, 3.0);
    lms.y = pow(lms.y, 3.0);
    lms.z = pow(lms.z, 3.0);

    // LMS -> linear RGB
    let rgb = vec3f(
          4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
        - 1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
        - 0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z,
    );

    // Calculate alpha for antialiasing at the edge of the sRGB gamut.
    let dx = dpdx(rgb);
    let dy = dpdy(rgb);
    let raw_gradient_magnitude = length(dx) + length(dy);
    let gradient_magnitude = clamp(raw_gradient_magnitude, 0.0, 0.1);

    let feathering_pixels = 1.0;
    let margin = feathering_pixels * 0.25 * gradient_magnitude;

    let lower_bound_diff = abs(rgb - vec3f(0.0));
    let upper_bound_diff = abs(vec3f(1.0) - rgb);

    let min_diff_1 = min(lower_bound_diff, upper_bound_diff);
    let min_diff = min(min(min_diff_1.r, min_diff_1.g), min_diff_1.b);

    let alpha = smoothstep(0.0, 1.0, min_diff / margin);

    // Out-of-gamut colors have zero alpha.
    if (any(rgb < vec3f(0.0)) || any(rgb > vec3f(1.0))) {
        return vec4f(rgb, 0.0);
    }

    return vec4f(clamp(rgb, vec3f(0.0), vec3f(1.0)), clamp(alpha, 0.0, 1.0));
}

// Antialiased ring, measured in real pixels. Must match `RING_WIDTH` in color_picker.rs.
fn ring(pixel_uv: vec2<f32>, half_size: vec2<f32>) -> f32 {
    const WIDTH: f32 = 28.0; // pixels

    let outer_radius = half_size.x;
    let inner_radius = half_size.x - WIDTH;
    let smoothness = 1.0;

    let r = length(pixel_uv);
    let inner = smoothstep(inner_radius - smoothness, inner_radius + smoothness, r);
    let outer = 1.0 - smoothstep(outer_radius - smoothness, outer_radius + smoothness, r);
    return inner * outer;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.uv.x;
    let v = in.uv.y;
    // Rust passes hue in radians; normalize to [0, 1] like the original did.
    let selected_hue = data.hue / (2.0 * PI);

    // hue wheel
    if (data.mode < 0.5) {
        let ring_mask = ring(in.pixel_uv, in.half_size);

        if (ring_mask > 0.0) {
            // uv.y points down here (vs. the original's y-up), so use +v.
            let hue = atan2(u, v) / (2.0 * PI);

            // magic values chosen so the whole wheel stays inside the sRGB gamut
            let hcl = vec3f(hue, 0.1254, 0.75);

            let color = hcl_rgb_with_alpha(hcl);
            return vec4f(color.rgb, ring_mask);
        }

        discard;
    }
    // main square
    else {
        // convert uv from [-1, 1] (y down) back to [0, 1] with chroma high at the top
        let uv01 = (in.uv + 1.0) / 2.0;

        let chroma = (1.0 - uv01.y) * 0.33;
        let lightness = uv01.x;

        let hcl = vec3(selected_hue, chroma, lightness);

        return hcl_rgb_with_alpha(hcl);
    }
}
