@group(0) @binding(0)
var<uniform> screen_resolution: vec2f;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) xs: vec2f,
    @location(1) ys: vec2f,
    @location(2) color: vec4f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var i_x = u32( in.index == 0 || in.index >= 4 );
    var i_y = u32( in.index % 2 );

    var x = in.xs[i_x];
    var y = in.ys[i_y];

    var out: VertexOutput;
    out.clip_position = vec4(x, y, 0.0, 1.0);

    // unit: screen pixel
    // zero: center of the rectangle
    out.uv.x = (f32 (2 * i_x) - 1.0) * screen_resolution.x / 2.0;
    out.uv.y = (f32 (2 * i_y) - 1.0) * screen_resolution.y / 2.0;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.uv;

    var radius = 20.0;
    var q = abs(pos) - vec2(screen_resolution.x / 2.0 - 10.0, screen_resolution.y / 2.0 - 10.0) + radius;

    var dist = length(max(q, vec2(0.0, 0.0))) - radius;

    var alpha = in.color.a * (1.0 - smoothstep(-1.0, 1.0, dist));
    return vec4(in.color.rgb, alpha);
}
