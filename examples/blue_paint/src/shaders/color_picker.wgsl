const PI: f32 = 3.141592653589793;
const color = vec3<f32>(0.2, 0.3, 0.8);

struct BaseUniforms {
    @location(1) window_size: vec2f,
    @location(0) t: f32,
};

@group(0) @binding(0)
var<uniform> base_unif: BaseUniforms;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) xs: vec2<f32>,
    @location(1) ys: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2f,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // triangle strip indexes
    let i_x = u32(in.index % 2);
    let i_y = u32(in.index >= 2);

    let x = in.xs[i_x];
    let y = in.ys[i_y];
    let clip_position = vec4f(x, y, 0.0, 1.0);

    let rect = vec2f(in.xs[1] - in.xs[0], in.ys[1] - in.ys[0]);

    let width = rect.x * base_unif.window_size.x;
    let height = rect.y * base_unif.window_size.y;
    let aspect = width / height;

    let u = f32(i_x);
    let v = f32(i_y);

    let uv = vec2f(u, v);

    return VertexOutput(clip_position, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x = in.uv.x;
    var y = in.uv.y;

    // Calculate the distance from the center of the UV space
    let center = vec2<f32>(0.5, 0.5);
    let uv = vec2<f32>(x, y);
    let distance = length(uv - center);

    // Define the inner and outer radius of the ring
    let innerRadius = 0.4;
    let outerRadius = 0.5;

    // Define the smoothing range for antialiasing
    let smoothness = 0.005;

    // Compute the smooth transition for the ring's alpha
    let ringAlpha = smoothstep(innerRadius, innerRadius + smoothness, distance) * 
                    (1.0 - smoothstep(outerRadius - smoothness, outerRadius, distance));

    // Base colors
    let ringColor = vec4<f32>(x, 0.0, y, 1.0);   // Inside the ring
    let greyColor = vec4<f32>(0.1, 0.1, 0.1, 1.0); // Outside the ring

    // Blend colors based on ringAlpha
    let color = mix(greyColor, ringColor, ringAlpha);

    return color;
}

