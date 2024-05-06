struct Uniforms {
    @location(1) screen_resolution: vec2f,
    @location(0) t: f32,
};

@group(0) @binding(0)
var<uniform> unif: Uniforms;

// has to match Rectangle
struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) xs: vec2f,
    @location(1) ys: vec2f,
    @location(2) color: vec4f,
    @location(3) last_hover: f32,
    @location(4) last_click: f32,
    @location(5) clickable: u32,
    @location(6) z: f32,
    @location(7) radius: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) dark: f32,
    @location(4) radius: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var i_x = u32( in.index == 0 || in.index >= 4 );
    var i_y = u32( in.index % 2 );

    // 0 <--> -1
    var x = in.xs[i_x];
    var y = in.ys[i_y];

    var clip_position = vec4(x, y, in.z, 1.0);

    var half_size = vec2f( 
        (in.xs[1] - in.xs[0]) * unif.screen_resolution.x / 2.0, 
        (in.ys[1] - in.ys[0]) * unif.screen_resolution.y / 2.0, 
    );

    // calculate for corners, will be interpolated.
    // interpolation after the abs() won't work.
    var corner = 2.0 * vec2f(vec2u(i_x, i_y)) - 1.0;    
    var uv = corner * half_size;

    var t_since_hover = (unif.t - in.last_hover) * 4.5;
    var hover = (1.0 - clamp(t_since_hover, 0.0, 1.0)) * f32(t_since_hover < 1.0) * f32(in.clickable);
    var t_since_click = (unif.t - in.last_click) * 4.1;
    var click = (1.0 - clamp(t_since_click, 0.0, 1.0)) * f32(t_since_click < 1.0) * f32(in.clickable);

    var dark_hover = 1.0 - hover * 0.32;
    var dark_click = 1.0 - click * 0.78;

    var dark = min(dark_click, dark_hover);
    return VertexOutput(clip_position, uv, half_size, in.color, dark, in.radius);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // in.uv: absolute value coords: 
    // +L <-- 0 --> +L
    // where L = rect_half_size (pixels)

    // todo: what the fuck is a q?
    var q = abs(in.uv) - in.half_size + in.radius;

    var dist = length(max(q, vec2(0.0, 0.0))) - in.radius;

    var alpha = in.color.a * (1.0 - smoothstep(-1.0, 1.0, dist));

    return vec4(in.color.rgb * in.dark, alpha);
}
