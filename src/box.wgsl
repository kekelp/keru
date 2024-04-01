@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

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
    out.uv.x = (f32 (i_x) - 0.5) * 2.0;
    out.uv.y = (f32 (i_y) - 0.5) * 2.0;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    var pos_rect = in.clip_position.xy * 2.0;
    var resolution = vec2(1200.0, 800.0);

    // scale (no translation doe) 
    var pos = (2.0 * in.clip_position.xy - resolution) / resolution.y;

    // var dist = length(pos);
    var radius = 0.05;
    var q = abs(pos) - vec2(0.5, 0.5) + radius;
    // var dist = max(q.x, q.y);
    var dist = length(max(q, vec2(0.0, 0.0))) - radius;

    // return vec4(dist, dist, dist, 1.0);

    var alpha = 1.0 - smoothstep(-1.0, 1.0, dist * resolution.y);
    return vec4(1.0-alpha, 0.0, alpha, 1.0);
}

fn box(position: vec2<f32>, halfSize: vec2<f32>, cornerRadius: f32) -> f32 {
    var pos2 = abs(position) - halfSize + cornerRadius;
    var d = length(max(pos2, vec2<f32>(0.0))) + min(max(pos2.x, pos2.y), 0.0) - cornerRadius;
    return d;
}
