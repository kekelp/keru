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

const CLICK_ANIMATION_FLAG: u32 = u32(1) << u32(8);
const OUTLINE_ONLY_FLAG: u32    = u32(1) << u32(9);
const HOVERED_FLAG: u32         = u32(1) << u32(10);

const SHAPE_RECTANGLE:   u32 = u32(0);
const SHAPE_CIRCLE: u32 = u32(1);
const SHAPE_RING:   u32 = u32(2);

struct RenderRect {
    @builtin(vertex_index) index: u32,

    @location(0) xs: vec2<f32>,               // Corresponds to rect.x_min, rect.y_min
    @location(1) ys: vec2<f32>,               // Corresponds to rect.x_max, rect.y_max
    @location(2) tex_coord_xs: vec2<f32>,     // Corresponds to tex_coords.x_min, tex_coords.y_min
    @location(3) tex_coord_ys: vec2<f32>,     // Corresponds to tex_coords.x_max, tex_coords.y_max

    @location(4) vertex_colors_tl: vec4<u32>, // Corresponds to vertex_colors[0]
    @location(5) vertex_colors_tr: vec4<u32>, // Corresponds to vertex_colors[1]
    @location(6) vertex_colors_bl: vec4<u32>, // Corresponds to vertex_colors[2]
    @location(7) vertex_colors_br: vec4<u32>, // Corresponds to vertex_colors[3]

    @location(8) z: f32,                      // Corresponds to z
    @location(9) last_hover: f32,             // Corresponds to last_hover
    @location(10) last_click: f32,            // Corresponds to last_click
    // todo: rename to shape_data and hope we never need more than one float
    @location(11) radius: f32,                // Corresponds to radius

    @location(12) flags: u32,                 // Corresponds to flags
    @location(13) _padding: u32,              // Corresponds to _padding

    @location(14) clip_xs: vec2<f32>,               // Corresponds to rect.x_min, rect.y_min
    @location(15) clip_ys: vec2<f32>,               // Corresponds to rect.x_max, rect.y_max
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) dark: f32,
    @location(4) radius: f32,
    @location(5) filled: f32,
    @location(6) tex_coords: vec2<f32>,
    @location(7) shape: u32,
    @location(8) corners: u32,
}

fn read_flag(value: u32, flag: u32) -> bool {
    return (value & flag) != 0u;
}

fn read_shape(flags: u32) -> u32 {
    return flags & 0x000000FF;
}

fn read_rounded_corners(flags: u32) -> u32 {
    return (flags >> 11) & 0xF; // Extract bits 11–14
}

@vertex
fn vs_main(in: RenderRect) -> VertexOutput {
    let i_x = u32( in.index == 0 || in.index >= 4 );
    let i_y = u32( in.index % 2 );

    // 0 <--> -1
    let x = in.xs[i_x];
    let y = in.ys[i_y];

    let x_clipped = clamp(x, in.clip_xs[0], in.clip_xs[1]);
    let y_clipped = clamp(y, in.clip_ys[0], in.clip_ys[1]);

    var vertex_colors = array(in.vertex_colors_bl, in.vertex_colors_tl, in.vertex_colors_br, in.vertex_colors_tr);
    let i_0123 = i_y + 2 * i_x;
    let color = vec4f(vertex_colors[i_0123]) / 255.0;

    let clip_position = vec4(x_clipped, y_clipped, in.z, 1.0);

    let half_size = vec2f( 
        (in.xs[1] - in.xs[0]) * unif.screen_resolution.x / 2.0, 
        (in.ys[1] - in.ys[0]) * unif.screen_resolution.y / 2.0, 
    );

    let uv_01 = vec2f(vec2u(i_x, i_y));
    
    let tex_coords = vec2<f32>(in.tex_coord_xs[i_x], in.tex_coord_ys[i_y]);

    let x_clip_factor = (x_clipped - x) / (in.xs[1] - in.xs[0]);
    let y_clip_factor = (y_clipped - y) / (in.ys[1] - in.ys[0]);

    let tex_width = in.tex_coord_xs[1] - in.tex_coord_xs[0];
    let tex_height = in.tex_coord_ys[1] - in.tex_coord_ys[0];

    let tex_coords_clipped = vec2<f32>(
        tex_coords.x + x_clip_factor * tex_width,
        tex_coords.y + y_clip_factor * tex_height
    );

    var uv = (uv_01 * half_size * 2.0);

    uv.y += (y_clipped - y) * unif.screen_resolution.y;
    uv.x += (x_clipped - x) * unif.screen_resolution.x;

    let clickable = f32(read_flag(in.flags, CLICK_ANIMATION_FLAG));
    let filled = f32( ! read_flag(in.flags, OUTLINE_ONLY_FLAG));

    let shape = read_shape(in.flags);

    let t_since_hover = (unif.t - in.last_hover) * 10.0;
    var hover: f32;
    let hover_bool = read_flag(in.flags, HOVERED_FLAG);
    if hover_bool {
        hover = clamp(t_since_hover, 0.0, 1.0) * clickable;
    } else {
        hover = (1.0 - clamp(t_since_hover, 0.0, 1.0)) * f32(t_since_hover < 1.0) * clickable;
    }

    let t_since_click = (unif.t - in.last_click) * 4.1;
    let click = (1.0 - clamp(t_since_click, 0.0, 1.0)) * f32(t_since_click < 1.0) * clickable;

    let dark_hover = 1.0 - hover * 0.32;
    let dark_click = 1.0 - click * 0.78;

    let dark = min(dark_click, dark_hover);
    let corners = read_rounded_corners(in.flags);
    return VertexOutput(clip_position, uv, half_size, color, dark, in.radius, filled, tex_coords_clipped, shape, corners);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var centered_uv = in.uv - (in.half_size);

    let index = select(0u, 1u, centered_uv.x < 0.0) + select(0u, 2u, centered_uv.y < 0.0);
    let flag = (in.corners >> index) & 1u;
    let rounded = flag != 0u;

    var circle_uv = abs(centered_uv);
    circle_uv.y *= (in.half_size.x / in.half_size.y);

    var alpha = in.color.a;

    var radius = f32(rounded) * in.radius;

    if (in.shape == SHAPE_RECTANGLE) {
        let q = abs(centered_uv) - in.half_size + radius;
        let dist = length(max(q, vec2(0.0, 0.0))) - radius;
        var is_inside = (1.0 - smoothstep(-1.0, 1.0, dist));
        var is_in_outline = (1.0 - smoothstep(1.0, -1.0, dist + 8.0));
        if radius < 1.0 {
            is_inside = 1.0;
            is_in_outline = f32(any(centered_uv > in.half_size - vec2f(8.0)));
        }
        
        let outline_mask = max(in.filled, is_in_outline); // filled: always 1. not filled, 1 in the outline, 0 inside.
        alpha = alpha * (is_inside * outline_mask);
    }

    else if (in.shape == SHAPE_CIRCLE) {
        let circle_alpha = in.half_size.x - length(circle_uv);
        alpha = alpha * clamp(circle_alpha, 0.0, 1.0);
    }
    
    else if (in.shape == SHAPE_RING ) {
        let circle_alpha = in.half_size.x - length(circle_uv);
        let inner_ring_alpha = length(circle_uv) - (in.half_size.x - radius);
        let ring_alpha = min(inner_ring_alpha, circle_alpha);
        
        alpha = alpha * clamp(ring_alpha, 0.0, 1.0);
    }

    if alpha == 0.0 { discard; }

    var tex_color = textureSample(my_texture, my_sampler, in.tex_coords);
    var rect_color = vec4(in.color.rgb * in.dark, alpha);

    var final_color = tex_color * rect_color;

    return final_color;
    // return vec4f(centered_uv.x, 0.0, centered_uv.y, 1.0);
}
