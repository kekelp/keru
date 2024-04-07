// @group(0) @binding(0)
// var t_diffuse: texture_2d<f32>;
// @group(0) @binding(1)
// var s_diffuse: sampler;

@group(0) @binding(0)
var<uniform> screen_resolution: vec2f;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) xs: vec2f,
    @location(1) ys: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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
    // center: center of the rectangle
    out.uv.x = (f32 (2 * i_x) - 1.0) * screen_resolution.x / 2.0;
    out.uv.y = (f32 (2 * i_y) - 1.0) * screen_resolution.y / 2.0;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    // var screen_resolution = vec2(1200.0, 800.0);
    var aspect = screen_resolution.y / screen_resolution.x;


    // // scale (no translation doe) (which is right doe)
    var pos = in.uv;
    // pos.y = pos.y * aspect;
    // var pos = (2.0 * in.clip_position.xy - screen_resolution) / screen_resolution.y;

    // return vec4(in.uv.xy, 1.0, 1.0);
    // return vec4(pos, 1.0, 1.0);


    // var dist = length(pos);
    var radius = 50.0;
    var q = abs(pos) - vec2(screen_resolution.x / 2.0 - 10.0, screen_resolution.y / 2.0 - 10.0) + radius;
    // var dist = max(q.x, q.y);
    var dist = length(max(q, vec2(0.0, 0.0))) - radius;

    // return vec4(dist, dist, dist, 1.0);

    var alpha = 1.0 - smoothstep(-1.0, 1.0, dist);
    return vec4(1.0-alpha, 0.0, alpha, 1.0);
}
