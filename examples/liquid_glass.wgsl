struct Params {
    // Rect of the glass node, in normalized screen coordinates (0 to 1).
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    // Screen size in physical pixels.
    screen_w: f32,
    screen_h: f32,
    // Corner radius in physical pixels.
    radius: f32,
    scale: f32,
}

var<immediate> params: Params;

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

const SHADOW_MARGIN: f32 = 30.0;
const BEZEL_WIDTH: f32 = 30.0;
const REFRACTION: f32 = 1.4;
const DISPERSION: f32 = 0.12;
const BLUR_RADIUS: f32 = 3.0;
const BLUR_TAPS: i32 = 12;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

fn screen_size() -> vec2<f32> {
    return vec2<f32>(params.screen_w, params.screen_h);
}

// The quad covers the node's rect plus a margin for the drop shadow.
@vertex
fn vs_glass(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let u = f32(vertex_index & 1u);
    let v = f32((vertex_index >> 1u) & 1u);

    let margin = SHADOW_MARGIN * params.scale / screen_size();
    let x = mix(params.min_x - margin.x, params.max_x + margin.x, u);
    let y = mix(params.min_y - margin.y, params.max_y + margin.y, v);

    var out: VertexOutput;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

// Signed distance to a rounded box centered on the origin. Negative inside.
fn sd_round_box(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Average of a small disc of samples around `pixel`.
fn sample_blurred(pixel: vec2<f32>) -> vec3<f32> {
    let radius = BLUR_RADIUS * params.scale;
    var acc = vec3<f32>(0.0);
    for (var i = 0; i < BLUR_TAPS; i++) {
        let fi = f32(i);
        let angle = fi * 2.39996;
        let r = sqrt((fi + 0.5) / f32(BLUR_TAPS)) * radius;
        let p = pixel + vec2<f32>(cos(angle), sin(angle)) * r;
        acc += textureSampleLevel(tex, samp, p / screen_size(), 0.0).rgb;
    }
    return acc / f32(BLUR_TAPS);
}

@fragment
fn fs_glass(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = in.position.xy;

    let rect_min = vec2<f32>(params.min_x, params.min_y) * screen_size();
    let rect_max = vec2<f32>(params.max_x, params.max_y) * screen_size();
    let center = (rect_min + rect_max) * 0.5;
    let half_size = (rect_max - rect_min) * 0.5;
    let radius = min(params.radius, min(half_size.x, half_size.y));

    let local = pixel - center;
    let d = sd_round_box(local, half_size, radius);

    // Drop shadow, offset a bit downwards.
    let shadow_d = sd_round_box(local - vec2<f32>(0.0, 8.0 * params.scale), half_size, radius);
    let shadow = 0.35 * (1.0 - smoothstep(-6.0 * params.scale, SHADOW_MARGIN * params.scale, shadow_d));

    // Anti-aliased coverage of the glass shape.
    let coverage = clamp(0.5 - d, 0.0, 1.0);

    // Outward normal of the shape, from the gradient of the distance field.
    let e = 0.5;
    let grad = vec2<f32>(
        sd_round_box(local + vec2<f32>(e, 0.0), half_size, radius) - sd_round_box(local - vec2<f32>(e, 0.0), half_size, radius),
        sd_round_box(local + vec2<f32>(0.0, e), half_size, radius) - sd_round_box(local - vec2<f32>(0.0, e), half_size, radius),
    );
    let n = grad / max(length(grad), 1e-5);

    // Refraction: the bezel near the edge acts like a convex lens and bends the light towards the center. The flat middle part does not bend it.
    let bezel = BEZEL_WIDTH * params.scale;
    let t = clamp(-d / bezel, 0.0, 1.0);
    let bend = pow(1.0 - t, 3.0);
    let offset = -n * bend * bezel * REFRACTION;

    // Chromatic aberration: each color channel bends by a slightly different amount.
    let r = sample_blurred(pixel + offset * (1.0 - DISPERSION)).r;
    let g = sample_blurred(pixel + offset).g;
    let b = sample_blurred(pixel + offset * (1.0 + DISPERSION)).b;
    var color = vec3<f32>(r, g, b);

    // Light tint, so that the glass is visible on flat backgrounds.
    color = mix(color, vec3<f32>(1.0), 0.06);

    // Specular rim. The light comes from the top-left, and it reflects on the opposite corner too.
    let light = normalize(vec2<f32>(-1.0, -1.0));
    let rim = pow(1.0 - clamp(-d / (2.5 * params.scale), 0.0, 1.0), 2.0);
    let facing = abs(dot(n, light));
    color += vec3<f32>(rim * (0.15 + 0.6 * facing * facing));

    // Soft inner glow along the bezel.
    color += vec3<f32>(0.08 * bend * (0.5 + 0.5 * facing));

    // Premultiplied output: glass over shadow.
    let alpha = coverage + shadow * (1.0 - coverage);
    return vec4<f32>(color * coverage, alpha);
}
