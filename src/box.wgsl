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
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var i_x = u32( in.index == 0 || in.index >= 4 );
    var i_y = u32( in.index % 2 );

    // 0 <--> -1
    var x = in.xs[i_x];
    var y = in.ys[i_y];

    var size = vec2f( 
        (in.xs[1] - in.xs[0]) * screen_resolution.x, 
        (in.ys[1] - in.ys[0]) * screen_resolution.y, 
    );


    var out: VertexOutput;
    out.clip_position = vec4(x, y, 0.0, 1.0);

    // calculate for corners and interpolate
    var corner = vec2f (2.0 * f32(i_x), 2.0 * f32(i_y)) - 1.0;
    
    out.uv = corner * size / 2.0;
    out.size = size;
    out.color = in.color;

    return out;
    // return VertexOutput (uv, size, in.color);

}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.uv;

    var radius = 30.0;
    var q = abs(pos) - vec2(in.size.x / 2.0, in.size.y / 2.0) + radius;

    var dist = length(max(q, vec2(0.0, 0.0))) - radius;

    var alpha = in.color.a * (1.0 - smoothstep(-1.0, 1.0, dist));
    return vec4(in.color.rgb, alpha);
}
