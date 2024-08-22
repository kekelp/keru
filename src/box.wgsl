struct Uniforms {
    @location(1) screen_resolution: vec2f,
    @location(0) t: f32,
};

@group(0) @binding(0)
var<uniform> unif: Uniforms;
@group(0) @binding(1)
var my_texture: texture_2d<f32>;
@group(0) @binding(2)
var my_sampler: sampler;

// has to match RenderRect
struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) xs: vec2f,
    @location(1) ys: vec2f,

    @location(2) tex_coord_xs: vec2f,
    @location(3) tex_coord_ys: vec2f,

    @location(4) vertex_colors_tl: vec4u,
    @location(5) vertex_colors_tr: vec4u,
    @location(6) vertex_colors_bl: vec4u,
    @location(7) vertex_colors_br: vec4u,
    @location(8) last_hover: f32,
    @location(9) last_click: f32,
    @location(10) clickable: u32,
    @location(11) z: f32,
    @location(12) radius: f32,
    @location(13) filled: u32,
    @location(14) _id: u32,

};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) dark: f32,
    @location(4) radius: f32,
    @location(5) filled: u32,
    @location(6) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let i_x = u32( in.index == 0 || in.index >= 4 );
    let i_y = u32( in.index % 2 );

    // 0 <--> -1
    let x = in.xs[i_x];
    let y = in.ys[i_y];

    var vertex_colors = array(in.vertex_colors_bl, in.vertex_colors_tl, in.vertex_colors_br, in.vertex_colors_tr);
    let i_1234 = i_y + 2 * i_x;
    let color = vec4f(vertex_colors[i_1234]) / 255.0;

    let clip_position = vec4(x, y, in.z, 1.0);

    let half_size = vec2f( 
        (in.xs[1] - in.xs[0]) * unif.screen_resolution.x / 2.0, 
        (in.ys[1] - in.ys[0]) * unif.screen_resolution.y / 2.0, 
    );

    let tex_coords = vec2<f32>(in.tex_coord_xs[i_x], in.tex_coord_ys[i_y]);

    // calculate for corners, will be interpolated.
    // interpolation after the abs() won't work.
    let corner = 2.0 * vec2f(vec2u(i_x, i_y)) - 1.0;    
    let uv = corner * half_size;

    let t_since_hover = (unif.t - in.last_hover) * 4.5;
    let hover = (1.0 - clamp(t_since_hover, 0.0, 1.0)) * f32(t_since_hover < 1.0) * f32(in.clickable);
    let t_since_click = (unif.t - in.last_click) * 4.1;
    let click = (1.0 - clamp(t_since_click, 0.0, 1.0)) * f32(t_since_click < 1.0) * f32(in.clickable);

    let dark_hover = 1.0 - hover * 0.32;
    let dark_click = 1.0 - click * 0.78;

    let dark = min(dark_click, dark_hover);
    return VertexOutput(clip_position, uv, half_size, color, dark, in.radius, in.filled, tex_coords);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // in.uv: absolute value coords: 
    // +L <-- 0 --> +L
    // where L = rect_half_size (pixels)

    // todo: what the fuck is a q?
    let q = abs(in.uv) - in.half_size + in.radius;

    let dist = length(max(q, vec2(0.0, 0.0))) - in.radius;

    let inside = (1.0 - smoothstep(-1.0, 1.0, dist));
    let outside = (1.0 - smoothstep(1.0, -1.0, dist + 8.0));

    let filled = f32(in.filled);
    let alpha = in.color.a * (inside * max(filled, outside));

    let tex_color = textureSample(my_texture, my_sampler, in.tex_coords);
    let rect_color = vec4(in.color.rgb * in.dark, alpha);
    // return tex_color;
    // return vec4(in.color.rgb * in.dark, alpha);
    return rect_color * tex_color;


}
