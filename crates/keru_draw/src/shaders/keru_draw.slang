import quad;

struct ResourceSlot {
    float4 a;
    float4 b;
    float4 c;
    float4 d;
}

struct PrimitiveSlot {
    float4 data[8];
}

struct ShapesGpu {
    StructuredBuffer<ResourceSlot> resources;
    StructuredBuffer<PrimitiveSlot> primitives;
    Texture2DArray<float4> atlas_texture;
    SamplerState atlas_sampler;
}

ParameterBlock<ShapesGpu> shapes;

struct GradientGpu {
    float4 color_start;
    float4 color_end;
    float2 p0; // gradient start (absolute screen coords); for radial: center
    float2 p1; // gradient end; for radial: p1.x = outer_radius, p1.y = inner_radius (flat region)
    uint gradient_type; // 0=solid, 1=linear, 2=radial
    uint _pad0;
    float2 _pad1;
}

struct RectangleGpu {
    float2 top_left;
    float2 size;
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    float corner_radius;
    float border_thickness;
    float2 gradient_direction; // unused (kept for layout compatibility)
    float4 color_start;        // unused
    float4 color_end;          // unused
    uint gradient_index; // index into gradients buffer
    uint rounded_corners; // bitflags: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right
    float2 texture_uv_origin; // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;   // pixel dimensions in atlas
    uint texture_page;        // atlas layer, 0xFFFFFFFF = no texture
    float blur_radius;
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    float _ns_pad1;
    float _ns_pad2;
    float _ns_pad3;
}

struct CircleGpu {
    float4 color_start;
    float4 color_end;
    float2 center;
    float2 radii;
    float2 angles;
    float2 gradient_direction; // unused (kept for layout compatibility)
    uint gradient_index; // index into gradients buffer
    uint texture_page;        // atlas layer, 0xFFFFFFFF = no texture
    float2 texture_uv_origin; // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;   // pixel dimensions in atlas
    float dash_length;        // 0 = no dashing, >0 = dash length in pixels
    float dash_offset;        // offset for dash pattern alignment
    float blur_radius;
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    float pie_stroke_thickness; // pie only: 0=filled, >0=hollow outline
    float pie_corner_radius;    // pie only: rounds center point and arc-edge corners
}

struct SegmentGpu {
    float2 start;
    float2 end;
    float4 color_start;  // unused
    float4 color_end;    // unused
    float4 thickness_dash; // [thickness, dash_length, dash_offset, stroke_thickness]
    uint gradient_index; // index into gradients buffer
    uint texture_page;        // atlas layer, 0xFFFFFFFF = no texture
    float2 texture_uv_origin; // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;   // pixel dimensions in atlas
    float blur_radius;
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    float4 _seg_pad;           // padding to 128 bytes (shared PrimitiveSlot size)
}

struct GridGpu {
    float4 color_start;  // unused
    float4 color_end;    // unused
    float2 top_left;
    float2 size;
    float2 offset;
    float2 lattice_size;
    float2 gradient_direction; // unused (kept for layout compatibility)
    float line_thickness;
    uint gradient_index; // index into gradients buffer
    uint grid_type; // 0=square, 1=hex
    uint texture_page;        // atlas layer, 0xFFFFFFFF = no texture
    float2 texture_uv_origin; // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;   // pixel dimensions in atlas
    float blur_radius;
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

struct TriangleGpu {
    float2 p0;
    float2 p1;
    float2 p2;
    float2 gradient_direction; // unused (kept for layout compatibility)
    float4 color_start;        // unused
    float4 color_end;          // unused
    uint gradient_index; // index into gradients buffer
    uint texture_page;        // atlas layer, 0xFFFFFFFF = no texture
    float2 texture_uv_origin; // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;   // pixel dimensions in atlas
    float blur_radius;
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    float stroke_thickness;    // 0 = filled, >0 = stroke only
    float corner_radius;
    float _tri_pad1;
    float _tri_pad2;
}

struct HexagonGpu {
    float2 center;
    float size;                // distance from center to vertex
    float rotation;            // rotation in radians
    float2 gradient_direction; // unused (kept for layout compatibility)
    float stroke_thickness;    // 0 = filled, >0 = stroke only
    uint texture_page;         // atlas layer, 0xFFFFFFFF = no texture
    float4 color_start;        // unused
    float4 color_end;          // unused
    uint gradient_index;       // index into gradients buffer
    float nine_slice_l;        // left inset in pixels (0 = no nine-slice)
    float2 texture_uv_origin;  // pixel coords in atlas (top-left corner)
    float2 texture_uv_size;    // pixel dimensions in atlas
    float blur_radius;
    float nine_slice_r;        // right inset in pixels
    float nine_slice_t;        // top inset in pixels
    float nine_slice_b;        // bottom inset in pixels
    uint nine_slice_tiling;    // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    float corner_radius;
    float4 _hex_pad;           // padding to 128 bytes (shared PrimitiveSlot size)
}

struct QuadraticBezierGpu {
    float2 p0;
    float2 p1;
    float2 p2;
    float thickness;
    float blur_radius;
    uint gradient_index; // index into gradients buffer (replaces color: float4, same 16 bytes)
    float stroke_thickness; // 0 = filled, >0 = stroke only
    float _cu2;
    float _cu3;
    float4 _qb_pad[5];         // padding to 128 bytes (shared PrimitiveSlot size)
}

struct Transform {
    float2 offset;
    float scale;
    float _padding;
}

struct ClipRect {
    float2 x_clip;
    float2 y_clip;
}

// Remap local_uv (0-1 over shape bounding box) to atlas-normalized UV using nine-slice rules.
// tiling_flags: bit 0=enabled; bits 1-2=x mode; bits 3-4=y mode
//   mode 0=stretch, 1=tile, 2=tile_fit (scale so a whole number of copies fit)
// Returns the atlas-space UV in [0,1].
float2 nine_slice_uv(
    float2 local_uv,
    float2 shape_size,
    float2 src_origin,
    float2 src_size,
    float inset_left, float inset_right, float inset_top, float inset_bottom,
    uint tiling_flags,
    float2 atlas_size
) {
    float pos_x = local_uv.x * shape_size.x;
    float pos_y = local_uv.y * shape_size.y;

    // Atlas-space boundaries of the stretchable middle region
    float src_mid_left   = src_origin.x + inset_left;
    float src_mid_right  = src_origin.x + src_size.x - inset_right;
    float src_mid_top    = src_origin.y + inset_top;
    float src_mid_bottom = src_origin.y + src_size.y - inset_bottom;

    // Screen-space start of the far-corner regions
    float dst_right  = shape_size.x - inset_right;
    float dst_bottom = shape_size.y - inset_bottom;

    uint tile_mode_x = (tiling_flags >> 1u) & 3u; // 0=stretch, 1=tile, 2=tile_fit
    uint tile_mode_y = (tiling_flags >> 3u) & 3u;

    // Region: 0=near corner, 1=middle, 2=far corner
    uint region_x = pos_x < inset_left ? 0u : (pos_x >= dst_right  ? 2u : 1u);
    uint region_y = pos_y < inset_top  ? 0u : (pos_y >= dst_bottom ? 2u : 1u);

    float tex_x;
    if (region_x == 0u) {
        tex_x = src_origin.x + pos_x;
    } else if (region_x == 2u) {
        tex_x = src_mid_right + (pos_x - dst_right);
    } else {
        float src_mid_width = src_mid_right - src_mid_left;
        float dst_mid_width = dst_right - inset_left;
        float t = (pos_x - inset_left) / max(dst_mid_width, 0.001);
        if (tile_mode_x == 1u && src_mid_width > 0.001) {
            tex_x = src_mid_left + fmod(t * dst_mid_width, src_mid_width);
            tex_x = clamp(tex_x, src_mid_left + 0.5, src_mid_right - 0.5);
        } else if (tile_mode_x == 2u && src_mid_width > 0.001) {
            float n = max(1.0, round(dst_mid_width / src_mid_width));
            tex_x = src_mid_left + fmod(t * n, 1.0) * src_mid_width;
            tex_x = clamp(tex_x, src_mid_left + 0.5, src_mid_right - 0.5);
        } else {
            tex_x = src_mid_left + t * src_mid_width;
        }
    }

    float tex_y;
    if (region_y == 0u) {
        tex_y = src_origin.y + pos_y;
    } else if (region_y == 2u) {
        tex_y = src_mid_bottom + (pos_y - dst_bottom);
    } else {
        float src_mid_height = src_mid_bottom - src_mid_top;
        float dst_mid_height = dst_bottom - inset_top;
        float t = (pos_y - inset_top) / max(dst_mid_height, 0.001);
        if (tile_mode_y == 1u && src_mid_height > 0.001) {
            tex_y = src_mid_top + fmod(t * dst_mid_height, src_mid_height);
            tex_y = clamp(tex_y, src_mid_top + 0.5, src_mid_bottom - 0.5);
        } else if (tile_mode_y == 2u && src_mid_height > 0.001) {
            float n = max(1.0, round(dst_mid_height / src_mid_height));
            tex_y = src_mid_top + fmod(t * n, 1.0) * src_mid_height;
            tex_y = clamp(tex_y, src_mid_top + 0.5, src_mid_bottom - 0.5);
        } else {
            tex_y = src_mid_top + t * src_mid_height;
        }
    }

    return float2(tex_x, tex_y) / atlas_size;
}

void box_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var box: RectangleGpu = reinterpret<RectangleGpu>(shapes.primitives[instanceIndex]);
    float blur_margin = box.blur_radius * 3.0;
    float2 expanded_top_left = box.top_left - float2(blur_margin, blur_margin);
    float2 expanded_size = box.size + float2(blur_margin * 2.0, blur_margin * 2.0);
    quad_vertex(expanded_top_left, expanded_size, vertexIndex, resolution, position, uv);
    screen_pos = expanded_top_left + uv * expanded_size;
}

float linear(float a, float b, float x) { return (x-a)/(b-a); }
float linearstep(float a, float b, float x) { return clamp(linear(a,b,x), 0.0, 1.0); }
float annular(float dist, float thickness) { return abs(dist + thickness * 0.5) - thickness * 0.5; }

// LOD for texture blur matching the shape's blur_radius (in screen pixels).
// tex_uv_size is in atlas pixels; shape_size is in screen pixels.
float tex_blur_lod(float blur_radius, float2 tex_uv_size, float2 shape_size) {
    float scale = max(tex_uv_size.x / max(shape_size.x, 0.001), tex_uv_size.y / max(shape_size.y, 0.001));
    return max(0.0, log2(blur_radius * scale));
}

float srgb_to_linear(float x) {
    return x <= 0.04045 ? x / 12.92 : pow((x + 0.055) / 1.055, 2.4);
}
float3 srgb_to_linear3(float3 srgb) {
    return float3(
        srgb_to_linear(srgb.r),
        srgb_to_linear(srgb.g),
        srgb_to_linear(srgb.b)
    );
}
float4 srgb_to_linear4(float4 srgba) {
    return float4(srgb_to_linear3(srgba.rgb), srgba.a);
}

// Oklab color space functions for perceptually uniform gradients
float3 linear_srgb_to_oklab(float3 c) {
    float l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    float m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    float s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    float l_ = pow(l, 1.0/3.0);
    float m_ = pow(m, 1.0/3.0);
    float s_ = pow(s, 1.0/3.0);

    return float3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    );
}

float3 oklab_to_linear_srgb(float3 c) {
    float l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    float m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    float s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;

    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;

    return float3(
        +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    );
}

// Interpolate between two colors in Oklab space
float4 gradient_oklab(float4 color_start, float4 color_end, float t) {
    float3 lab_start = linear_srgb_to_oklab(color_start.rgb);
    float3 lab_end = linear_srgb_to_oklab(color_end.rgb);
    float3 lab_mixed = lerp(lab_start, lab_end, t);
    float3 rgb = oklab_to_linear_srgb(lab_mixed);
    float alpha = lerp(color_start.a, color_end.a, t);
    return float4(rgb, alpha);
}

// Per-corner rounded box SDF
// r.x = top-right, r.y = bottom-right, r.z = bottom-left, r.w = top-left
float sd_rounded_box(float2 p, float2 size, float4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.wz;
    r.x = (p.y > 0.0) ? r.x : r.y;
    float2 q = abs(p) - size + r.x;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r.x;
}

float4 box_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool surface_is_srgb, float transform_scale): SV_Target {
    var box: RectangleGpu = reinterpret<RectangleGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    float blur_margin = box.blur_radius * 3.0;
    float2 expanded_top_left = box.top_left - float2(blur_margin, blur_margin);
    float2 expanded_size = box.size + float2(blur_margin * 2.0, blur_margin * 2.0);
    float2 p = expanded_top_left + uv * expanded_size - (box.top_left + box.size * 0.5);

    // Compute per-corner radii based on bitflags
    // Bits: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right (screen space, Y down)
    // SDF expects: x=top-right, y=bottom-right, z=bottom-left, w=top-left (math space, Y up)
    // Screen Y is flipped relative to SDF Y, so we swap top<->bottom
    float r = box.corner_radius;
    float4 corner_radii = float4(
        (box.rounded_corners & 8u) != 0u ? r : 0.0,
        (box.rounded_corners & 2u) != 0u ? r : 0.0,
        (box.rounded_corners & 1u) != 0u ? r : 0.0,
        (box.rounded_corners & 4u) != 0u ? r : 0.0
    );

    float2 half_size = box.size * 0.5;
    float dist = sd_rounded_box(p, half_size, corner_radii);

    if (box.border_thickness > 0.001) {
        dist = annular(dist, box.border_thickness);
    }

    float alpha;
    if (box.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / box.blur_radius));
    } else {
        float aa_width = 0.5 / max(transform_scale, 0.001);
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient/solid color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[box.gradient_index]);
    float2 pixel_pos_box = p + box.top_left + box.size * 0.5;
    float4 color;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(pixel_pos_box - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(pixel_pos_box - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (surface_is_srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (box.texture_page != 0xFFFFFFFFu) {
        // Get atlas dimensions
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));

        float2 local_uv = (p + box.size * 0.5) / box.size;
        float2 tex_uv;
        if ((box.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(local_uv, box.size, box.texture_uv_origin, box.texture_uv_size,
                box.nine_slice_l, box.nine_slice_r, box.nine_slice_t, box.nine_slice_b,
                box.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (box.texture_uv_origin + box.texture_uv_size * local_uv) / atlas_size;
        }

        float lod = tex_blur_lod(box.blur_radius, box.texture_uv_size, box.size);

        // Sample texture
        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(box.texture_page)),
            lod
        );

        // Atlas is Rgba8Unorm (linear read); linearize sRGB-encoded image data when surface is sRGB.
        if (surface_is_srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }

        // Multiply texture color with fill color (tinting)
        color = tex_color * color;
    }

    return float4(color.rgb, color.a * alpha);
}

void circle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var circle: CircleGpu = reinterpret<CircleGpu>(shapes.primitives[instanceIndex]);

    // Create bounding box for the circle, expanded for blur falloff and pie corner rounding.
    float outer_radius = circle.radii.y;
    float blur_margin = circle.blur_radius * 3.0;
    float expand = blur_margin + circle.pie_corner_radius;
    float2 top_left = circle.center - float2(outer_radius + expand, outer_radius + expand);
    float2 size = float2((outer_radius + expand) * 2.0, (outer_radius + expand) * 2.0);

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

float normalize_angle(float angle) {
    const float TAU = 6.28318530718;
    float result = fmod(angle, TAU);
    if (result < 0.0) result += TAU;
    return result;
}


float sd_circle_variant(float2 p, CircleGpu circle) {
    const float TAU = 6.28318530718;

    float inner_radius = circle.radii.x;
    float outer_radius = circle.radii.y;
    float start_angle = circle.angles.x;
    float end_angle = circle.angles.y;

    float dist_from_center = length(p);

    // Check if this is a full circle (angles span full range)
    float angle_span = abs(end_angle - start_angle);
    bool is_full_circle = abs(angle_span - TAU) < 0.01;

    if (is_full_circle) {
        return dist_from_center - outer_radius;
    }

    // Normalize the pixel's angle relative to start_angle, into [0, span]
    float angle = atan2(p.y, p.x);
    float angle_rel = normalize_angle(angle - start_angle);
    float span = normalize_angle(end_angle - start_angle);

    if (inner_radius < 0.001) {
        // Pie slice - proper IQ sdPie, rotated to handle arbitrary start/end angles.
        // Normalize span into (0, TAU].
        float span = normalize_angle(end_angle - start_angle);
        if (span < 0.001) span = TAU;

        if (abs(span - TAU) < 0.01) {
            // Full circle
            return dist_from_center - outer_radius;
        }

        // Rotate p so the arc midpoint aligns with +Y.
        float mid_angle = start_angle + span * 0.5;
        float rot = 1.5707963268 - mid_angle; // PI/2 - mid_angle
        float cr = cos(rot);
        float sr = sin(rot);
        float2 rp = float2(p.x * cr - p.y * sr, p.x * sr + p.y * cr);

        // IQ sdPie: c = (sin, cos) of half-aperture.
        float half_span = span * 0.5;
        float2 c = float2(sin(half_span), cos(half_span));
        rp.x = abs(rp.x);
        float l = length(rp) - outer_radius;
        float m = length(rp - c * clamp(dot(rp, c), 0.0, outer_radius));
        return max(l, m * sign(c.y * rp.x - c.x * rp.y));
    } else {
        // Arc: nearest point on arc centerline (proper SDF).
        // clamp() alone is wrong for the start cap: angle_rel just before start wraps to ~2π,
        // which clamps to span (the end cap) instead of 0 (the start cap). Pick the nearer cap.
        float mid_radius = (inner_radius + outer_radius) * 0.5;
        float half_thickness = (outer_radius - inner_radius) * 0.5;
        float nearest_rel;
        if (angle_rel <= span) {
            nearest_rel = angle_rel;
        } else {
            float dist_to_start = 6.28318530718 - angle_rel; // gap going back to start
            float dist_to_end   = angle_rel - span;           // gap going forward to end
            nearest_rel = (dist_to_start < dist_to_end) ? 0.0 : span;
        }
        float2 nearest_on_arc = mid_radius * float2(cos(start_angle + nearest_rel), sin(start_angle + nearest_rel));
        return length(p - nearest_on_arc) - half_thickness;
    }
}

float4 circle_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool srgb, float transform_scale): SV_Target {
    var circle: CircleGpu = reinterpret<CircleGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Convert UV to position relative to circle center, accounting for blur-expanded bounding quad
    float outer_radius = circle.radii.y;
    float inner_radius = circle.radii.x;
    float blur_margin = circle.blur_radius * 3.0;
    float expand = blur_margin + circle.pie_corner_radius;
    float2 p = (uv - 0.5) * float2((outer_radius + expand) * 2.0);

    float dist = sd_circle_variant(p, circle);

    // Apply ring transform for full circles with an inner radius; arcs already encode thickness in their SDF.
    float angle_span = abs(circle.angles.y - circle.angles.x);
    bool is_full_circle = abs(angle_span - 6.28318530718) < 0.01;
    if (is_full_circle && inner_radius > 0.001) {
        dist = annular(dist, outer_radius - inner_radius);
    }

    // Pie-specific rounding and stroke.
    if (inner_radius < 0.001) {
        if (circle.pie_corner_radius > 0.001) {
            dist -= circle.pie_corner_radius;
        }
        if (circle.pie_stroke_thickness > 0.001) {
            dist = annular(dist, circle.pie_stroke_thickness);
        }
    }

    float aa_width = 0.5 / max(transform_scale, 0.001);
    float alpha;
    if (circle.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / circle.blur_radius));
    } else {
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    // dashed circle
    if (circle.dash_length > 0.0001) {
        // Use the middle radius for arc length calculation
        float mid_radius = (circle.radii.x + circle.radii.y) * 0.5;
        if (mid_radius < 0.001) mid_radius = circle.radii.y; // for filled circles, use outer radius

        // Get angle in [0, 2π], then offset by start angle so dashes begin at arc start
        float angle = atan2(p.y, p.x);
        if (angle < 0.0) angle += 6.28318530718;
        angle = angle - circle.angles.x;

        // Convert angle to arc length position, apply dash offset
        float arc_pos = angle * mid_radius + circle.dash_offset;

        // Apply dashing pattern (similar to segment)
        float dash_period = circle.dash_length * 2.0;
        float pos_in_period = fmod(arc_pos, dash_period);
        if (pos_in_period < 0.0) pos_in_period += dash_period;

        // Signed distance to nearest dash boundary: positive inside a dash, negative in a gap.
        // min(pos_in_period, ...) alone is wrong near the start of the next dash: it only
        // measures distance to the end of the previous dash, giving a large negative in the gap
        // near the next start. Compute distance to both neighboring boundaries explicitly.
        float dist_to_nearest_edge;
        if (pos_in_period <= circle.dash_length) {
            dist_to_nearest_edge = min(pos_in_period, circle.dash_length - pos_in_period);
        } else {
            float dist_to_prev = pos_in_period - circle.dash_length;
            float dist_to_next = dash_period - pos_in_period;
            dist_to_nearest_edge = -min(dist_to_prev, dist_to_next);
        }

        float dash_alpha;
        if (circle.blur_radius > 0.5) {
            dash_alpha = 1.0 / (1.0 + exp(1.7 * (-dist_to_nearest_edge) / circle.blur_radius));
        } else {
            dash_alpha = linearstep(-aa_width, aa_width, dist_to_nearest_edge);
        }
        alpha *= dash_alpha;
    }

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[circle.gradient_index]);
    float2 pixel_pos_circle = p + circle.center;
    float4 color;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(pixel_pos_circle - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(pixel_pos_circle - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (circle.texture_page != 0xFFFFFFFFu) {
        // Get atlas dimensions
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));

        float2 shape_sz = float2(outer_radius * 2.0);
        float2 local_uv = (p + float2(outer_radius)) / shape_sz;
        float2 tex_uv;
        if ((circle.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(local_uv, shape_sz, circle.texture_uv_origin, circle.texture_uv_size,
                circle.nine_slice_l, circle.nine_slice_r, circle.nine_slice_t, circle.nine_slice_b,
                circle.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (circle.texture_uv_origin + circle.texture_uv_size * local_uv) / atlas_size;
        }

        float lod = tex_blur_lod(circle.blur_radius, circle.texture_uv_size, shape_sz);

        // Sample texture
        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(circle.texture_page)),
            lod
        );

        if (srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }

        // Multiply texture color with fill color (tinting)
        color = tex_color * color;
    }

    return float4(color.rgb, alpha * color.a);
}

void segment_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var seg: SegmentGpu = reinterpret<SegmentGpu>(shapes.primitives[instanceIndex]);

    // Create bounding box for the segment, expanded for blur falloff
    float half_thickness = seg.thickness_dash[0] * 0.5;
    float blur_margin = seg.blur_radius * 3.0;
    float2 min_point = min(seg.start, seg.end) - float2(half_thickness + blur_margin, half_thickness + blur_margin);
    float2 max_point = max(seg.start, seg.end) + float2(half_thickness + blur_margin, half_thickness + blur_margin);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// SDF for line segment from Inigo Quilez
float sd_segment(float2 p, float2 a, float2 b) {
    float2 pa = p - a;
    float2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

float4 segment_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool srgb, float transform_scale): SV_Target {
    var seg: SegmentGpu = reinterpret<SegmentGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Convert UV to world position, accounting for blur-expanded bounding quad
    float half_thickness = seg.thickness_dash[0] * 0.5;
    float blur_margin = seg.blur_radius * 3.0;
    float2 min_point = min(seg.start, seg.end) - float2(half_thickness + blur_margin, half_thickness + blur_margin);
    float2 max_point = max(seg.start, seg.end) + float2(half_thickness + blur_margin, half_thickness + blur_margin);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    float2 p = top_left + uv * size;

    float dist = sd_segment(p, seg.start, seg.end) - half_thickness;

    if (seg.thickness_dash[3] > 0.001) {
        dist = annular(dist, seg.thickness_dash[3]);
    }

    float aa_width = 0.5 / max(transform_scale, 0.001);
    float alpha;
    if (seg.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / seg.blur_radius));
    } else {
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    if (alpha < 0.001) {
        discard;
    }

    // Handle dashed lines
    float dash_length = seg.thickness_dash[1];
    if (dash_length > 0.0001) {
        float dash_offset = seg.thickness_dash[2];
        float2 seg_vec = seg.end - seg.start;
        float seg_len = length(seg_vec);
        float t = 0.0;
        if (seg_len > 0.001) {
            t = dot(p - seg.start, seg_vec) / seg_len;
        }
        t += dash_offset;

        float dash_period = dash_length * 2.0;
        float pos_in_period = fmod(t, dash_period);
        if (pos_in_period < 0.0) pos_in_period += dash_period;

        // Signed distance to nearest dash boundary: positive inside a dash, negative in a gap.
        // Same fix as circle dashes: compute distance to both neighboring boundaries.
        float dist_to_nearest_edge;
        if (pos_in_period <= dash_length) {
            dist_to_nearest_edge = min(pos_in_period, dash_length - pos_in_period);
        } else {
            float dist_to_prev = pos_in_period - dash_length;
            float dist_to_next = dash_period - pos_in_period;
            dist_to_nearest_edge = -min(dist_to_prev, dist_to_next);
        }

        float dash_alpha;
        if (seg.blur_radius > 0.5) {
            dash_alpha = 1.0 / (1.0 + exp(1.7 * (-dist_to_nearest_edge) / seg.blur_radius));
        } else {
            dash_alpha = linearstep(-0.5, 0.5, dist_to_nearest_edge);
        }
        alpha *= dash_alpha;

        if (alpha < 0.001) {
            discard;
        }
    }

    // Calculate gradient color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[seg.gradient_index]);
    float4 color;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(p - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(p - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (seg.texture_page != 0xFFFFFFFFu) {
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));
        float2 orig_min = min(seg.start, seg.end) - float2(half_thickness, half_thickness);
        float2 orig_size = max(seg.start, seg.end) + float2(half_thickness, half_thickness) - orig_min;
        float2 local_uv = (p - orig_min) / orig_size;
        float2 tex_uv;
        if ((seg.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(local_uv, orig_size, seg.texture_uv_origin, seg.texture_uv_size,
                seg.nine_slice_l, seg.nine_slice_r, seg.nine_slice_t, seg.nine_slice_b,
                seg.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (seg.texture_uv_origin + seg.texture_uv_size * local_uv) / atlas_size;
        }

        float lod = tex_blur_lod(seg.blur_radius, seg.texture_uv_size, orig_size);

        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(seg.texture_page)),
            lod
        );
        if (srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }
        color = tex_color * color;
    }

    return float4(color.rgb, alpha * color.a);
}

void grid_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var grid: GridGpu = reinterpret<GridGpu>(shapes.primitives[instanceIndex]);
    quad_vertex(grid.top_left, grid.size, vertexIndex, resolution, position, uv);
    screen_pos = grid.top_left + uv * grid.size;
}

// Thanks leftofzen. https://www.shadertoy.com/view/wtdSzX
float4 grid_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool surface_is_srgb, float transform_scale): SV_Target {
    var grid: GridGpu = reinterpret<GridGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    // Apply clipping
    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Transform to grid space
    float2 local_pos = uv * grid.size;
    float2 p_pixels = local_pos + grid.offset;

    float dist;
    if (grid.grid_type == 0) {
        // Rectangular grid
        float2 p = p_pixels / grid.lattice_size;
        float2 grid_pos = fmod(p, 1.0);
        float2 dist_to_line = min(grid_pos, 1.0 - grid_pos);
        float2 dist_pixels = dist_to_line * grid.lattice_size;
        dist = min(dist_pixels.x, dist_pixels.y);
    } else {
        const float COS30 = 0.8660254;
        float lattice_size = grid.lattice_size.x * COS30;

        // Scale by lattice_size to work in normalized hex space
        float2 p = p_pixels / lattice_size;
        float2 s = float2(1.0, 1.7320508); // vec2(1, sqrt(3))

        // Two candidate hexagon centers
        float2 hC1 = floor(p / s) + 0.5;
        float2 hC2 = floor((p - float2(0.5, 1.0)) / s) + 0.5;

        // Positions relative to each center
        float2 h1 = p - hC1 * s;
        float2 h2 = p - (hC2 + 0.5) * s;

        // Pick nearest center
        float2 h = dot(h1, h1) < dot(h2, h2) ? h1 : h2;

        // Hexagon SDF
        float2 q = abs(h);
        float hexDist = max(dot(q, s * 0.5), q.x);

        // Distance to hex edge (edge is at hexDist = 0.5)
        dist = abs(hexDist - 0.5) * lattice_size;
    }

    // Apply line thickness and antialiasing
    float line_width = grid.line_thickness / max(transform_scale, 0.001);
    float half_thickness = line_width * 0.5;
    float d = dist - half_thickness; // effective SDF: 0 at line edge, positive outside
    float alpha;
    if (grid.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * d / grid.blur_radius));
    } else {
        float aa_width = 0.5 / max(transform_scale, 0.001);
        alpha = 1.0 - linearstep(-aa_width, aa_width, d);
    }

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient/solid color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[grid.gradient_index]);
    float4 color;
    float2 p = uv * grid.size - grid.size * 0.5;
    float2 pixel_pos_grid = p + grid.top_left + grid.size * 0.5;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(pixel_pos_grid - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(pixel_pos_grid - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (surface_is_srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (grid.texture_page != 0xFFFFFFFFu) {
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));
        float2 tex_uv;
        if ((grid.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(uv, grid.size, grid.texture_uv_origin, grid.texture_uv_size,
                grid.nine_slice_l, grid.nine_slice_r, grid.nine_slice_t, grid.nine_slice_b,
                grid.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (grid.texture_uv_origin + grid.texture_uv_size * uv) / atlas_size;
        }
        float lod = tex_blur_lod(grid.blur_radius, grid.texture_uv_size, grid.size);

        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(grid.texture_page)),
            lod
        );
        if (surface_is_srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }
        color = tex_color * color;
    }

    return float4(color.rgb, color.a * alpha);
}

void triangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var tri: TriangleGpu = reinterpret<TriangleGpu>(shapes.primitives[instanceIndex]);

    // Create bounding box for the triangle, expanded for blur falloff and corner rounding
    float blur_margin = tri.blur_radius * 3.0;
    float tri_expand = blur_margin + tri.corner_radius;
    float2 min_point = min(tri.p0, min(tri.p1, tri.p2)) - float2(tri_expand, tri_expand);
    float2 max_point = max(tri.p0, max(tri.p1, tri.p2)) + float2(tri_expand, tri_expand);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// SDF for triangle - returns signed distance
float sd_triangle(float2 p, float2 p0, float2 p1, float2 p2) {
    float2 e0 = p1 - p0;
    float2 e1 = p2 - p1;
    float2 e2 = p0 - p2;

    float2 v0 = p - p0;
    float2 v1 = p - p1;
    float2 v2 = p - p2;

    float2 pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
    float2 pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
    float2 pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);

    float s = sign(e0.x * e2.y - e0.y * e2.x);
    float2 d = min(min(float2(dot(pq0, pq0), s * (v0.x * e0.y - v0.y * e0.x)),
                       float2(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x))),
                       float2(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x)));

    return -sqrt(d.x) * sign(d.y);
}

float4 triangle_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool surface_is_srgb, float transform_scale): SV_Target {
    var tri: TriangleGpu = reinterpret<TriangleGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Convert UV to world position, accounting for blur-expanded and corner-rounded bounding quad
    float blur_margin = tri.blur_radius * 3.0;
    float tri_expand = blur_margin + tri.corner_radius;
    float2 orig_min = min(tri.p0, min(tri.p1, tri.p2));
    float2 orig_max = max(tri.p0, max(tri.p1, tri.p2));
    float2 min_point = orig_min - float2(tri_expand, tri_expand);
    float2 max_point = orig_max + float2(tri_expand, tri_expand);
    float2 top_left = min_point;
    float2 size = max_point - min_point;
    float2 p = top_left + uv * size;

    float dist = sd_triangle(p, tri.p0, tri.p1, tri.p2);

    if (tri.corner_radius > 0.001) {
        dist -= tri.corner_radius;
    }
    if (tri.stroke_thickness > 0.001) {
        dist = annular(dist, tri.stroke_thickness);
    }

    float alpha;
    if (tri.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / tri.blur_radius));
    } else {
        float aa_width = 0.5 / max(transform_scale, 0.001);
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[tri.gradient_index]);
    float4 color;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(p - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(p - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (surface_is_srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (tri.texture_page != 0xFFFFFFFFu) {
        // Get atlas dimensions
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));

        float2 shape_sz = orig_max - orig_min;
        float2 local_uv = (p - orig_min) / shape_sz;
        float2 tex_uv;
        if ((tri.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(local_uv, shape_sz, tri.texture_uv_origin, tri.texture_uv_size,
                tri.nine_slice_l, tri.nine_slice_r, tri.nine_slice_t, tri.nine_slice_b,
                tri.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (tri.texture_uv_origin + tri.texture_uv_size * local_uv) / atlas_size;
        }

        float lod = tex_blur_lod(tri.blur_radius, tri.texture_uv_size, shape_sz);

        // Sample texture
        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(tri.texture_page)),
            lod
        );

        if (surface_is_srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }

        // Multiply texture color with fill color (tinting)
        color = tex_color * color;
    }

    return float4(color.rgb, color.a * alpha);
}

void hexagon_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var hex: HexagonGpu = reinterpret<HexagonGpu>(shapes.primitives[instanceIndex]);

    // Create bounding box for the hexagon, expanded for blur falloff and corner rounding
    float blur_margin = hex.blur_radius * 3.0;
    float extent = hex.size + blur_margin + hex.corner_radius;
    float2 top_left = hex.center - float2(extent, extent);
    float2 size = float2(extent * 2.0, extent * 2.0);

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// SDF for regular hexagon (flat-top orientation by default)
// r = distance from center to edge
float sd_hexagon(float2 p, float r) {
    const float3 k = float3(-0.866025404, 0.5, 0.577350269);
    p = abs(p);
    p -= 2.0 * min(dot(k.xy, p), 0.0) * k.xy;
    p -= float2(clamp(p.x, -k.z * r, k.z * r), r);
    return length(p) * sign(p.y);
}

float4 hexagon_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool surface_is_srgb, float transform_scale): SV_Target {
    var hex: HexagonGpu = reinterpret<HexagonGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Convert UV to position relative to hexagon center, accounting for blur-expanded bounding quad
    float blur_margin = hex.blur_radius * 3.0;
    float extent = (hex.size + blur_margin + hex.corner_radius) * 1.1547;
    float2 p = (uv - 0.5) * 2.0 * extent;

    // Apply rotation
    float cos_r = cos(hex.rotation);
    float sin_r = sin(hex.rotation);
    float2 rotated_p = float2(
        p.x * cos_r - p.y * sin_r,
        p.x * sin_r + p.y * cos_r
    );

    // Calculate distance to hexagon
    float dist = sd_hexagon(rotated_p, hex.size);

    if (hex.corner_radius > 0.001) {
        dist -= hex.corner_radius;
    }
    if (hex.stroke_thickness > 0.001) {
        dist = annular(dist, hex.stroke_thickness);
    }

    float alpha;
    if (hex.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / hex.blur_radius));
    } else {
        float aa_width = 0.5 / max(transform_scale, 0.001);
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[hex.gradient_index]);
    float2 pixel_pos_hex = p + hex.center;
    float4 color;
    if (grad.gradient_type == 1) {
        float2 dir = grad.p1 - grad.p0;
        float t = dot(pixel_pos_hex - grad.p0, dir) / max(dot(dir, dir), 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else if (grad.gradient_type == 2) {
        float t = (length(pixel_pos_hex - grad.p0) - grad.p1.y) / max(grad.p1.x - grad.p1.y, 0.001);
        color = gradient_oklab(grad.color_start, grad.color_end, clamp(t, 0.0, 1.0));
    } else {
        color = grad.color_start;
    }

    if (surface_is_srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    // Sample texture if enabled (texture_page != 0xFFFFFFFF)
    if (hex.texture_page != 0xFFFFFFFFu) {
        uint atlas_width, atlas_height, atlas_elements;
        shapes.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
        float2 atlas_size = float2(float(atlas_width), float(atlas_height));
        // Map p (in the 1.1547-scaled space) back to 0-1 over the original bounding box
        float2 shape_sz = float2(hex.size * 2.0);
        float2 local_uv = (p * 0.866025 + float2(hex.size)) / shape_sz;
        float2 tex_uv;
        if ((hex.nine_slice_tiling & 1u) != 0u) {
            tex_uv = nine_slice_uv(local_uv, shape_sz, hex.texture_uv_origin, hex.texture_uv_size,
                hex.nine_slice_l, hex.nine_slice_r, hex.nine_slice_t, hex.nine_slice_b,
                hex.nine_slice_tiling, atlas_size);
        } else {
            tex_uv = (hex.texture_uv_origin + hex.texture_uv_size * local_uv) / atlas_size;
        }
        float lod = tex_blur_lod(hex.blur_radius, hex.texture_uv_size, shape_sz);

        float4 tex_color = shapes.atlas_texture.SampleLevel(
            shapes.atlas_sampler,
            float3(tex_uv, float(hex.texture_page)),
            lod
        );
        if (surface_is_srgb) {
            tex_color = srgb_to_linear4(tex_color);
        }
        color = tex_color * color;
    }

    return float4(color.rgb, color.a * alpha);
}

void quadratic_bezier_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var seg: QuadraticBezierGpu = reinterpret<QuadraticBezierGpu>(shapes.primitives[instanceIndex]);

    // Create bounding box for the quadratic bezier, expanded for blur falloff
    float half_thickness = seg.thickness * 0.5;
    float blur_margin = seg.blur_radius * 3.0;
    float2 min_point = min(seg.p0, min(seg.p1, seg.p2)) - float2(half_thickness + blur_margin, half_thickness + blur_margin);
    float2 max_point = max(seg.p0, max(seg.p1, seg.p2)) + float2(half_thickness + blur_margin, half_thickness + blur_margin);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// True quadratic Bezier SDF using analytic cubic solution
// B(t) = (1-t)²·p0 + 2(1-t)t·p1 + t²·p2
float sd_quadratic_bezier(float2 pos, float2 p0, float2 p1, float2 p2) {
    float2 a = p1 - p0;
    float2 b = p0 - 2.0 * p1 + p2;
    float2 c = a * 2.0;
    float2 d = p0 - pos;

    // Cubic coefficients for closest point: dot(B'(t), B(t) - pos) = 0
    float bb = dot(b, b);

    // Handle degenerate case (collinear points -> line segment)
    if (bb < 0.0001) {
        // Fallback to line segment p0 -> p2
        float2 e = p2 - p0;
        float t = clamp(dot(pos - p0, e) / (dot(e, e) + 0.0001), 0.0, 1.0);
        return length(pos - (p0 + e * t));
    }

    float kk = 1.0 / bb;
    float kx = kk * dot(a, b);
    float ky = kk * (2.0 * dot(a, a) + dot(d, b)) / 3.0;
    float kz = kk * dot(d, a);

    float p = ky - kx * kx;
    float q = kx * (2.0 * kx * kx - 3.0 * ky) + kz;
    float p3 = p * p * p;
    float h = q * q + 4.0 * p3;

    float res;

    if (h >= 0.0) {
        // One real root
        h = sqrt(h);
        float2 x = (float2(h, -h) - q) * 0.5;
        float2 uv = sign(x) * pow(abs(x), float2(1.0/3.0, 1.0/3.0));
        float t = clamp(uv.x + uv.y - kx, 0.0, 1.0);
        float2 qv = d + (c + b * t) * t;
        res = dot(qv, qv);
    } else {
        // Three real roots - need to check all
        float z = sqrt(-p);
        float v = acos(q / (p * z * 2.0)) / 3.0;
        float m = cos(v);
        float n = sin(v) * 1.732050808; // sqrt(3)

        float3 t = clamp(float3(m + m, -n - m, n - m) * z - kx, float3(0.0), float3(1.0));

        float2 qx = d + (c + b * t.x) * t.x;
        float2 qy = d + (c + b * t.y) * t.y;
        float2 qz = d + (c + b * t.z) * t.z;

        float dx = dot(qx, qx);
        float dy = dot(qy, qy);
        float dz = dot(qz, qz);

        res = min(dx, min(dy, dz));
    }

    return sqrt(res);
}

float4 quadratic_bezier_fragment(uint primitiveIndex, uint clipRectIndex, float2 uv, float2 screen_pos, bool srgb, float transform_scale): SV_Target {
    var seg: QuadraticBezierGpu = reinterpret<QuadraticBezierGpu>(shapes.primitives[primitiveIndex]);
    var clip: ClipRect = reinterpret<ClipRect>(shapes.resources[clipRectIndex].a);

    if (screen_pos.x < clip.x_clip.x || screen_pos.x > clip.x_clip.y ||
        screen_pos.y < clip.y_clip.x || screen_pos.y > clip.y_clip.y) {
        discard;
    }

    // Convert UV to world position, accounting for blur-expanded bounding quad
    float half_thickness = seg.thickness * 0.5;
    float blur_margin = seg.blur_radius * 3.0;
    float2 min_point = min(seg.p0, min(seg.p1, seg.p2)) - float2(half_thickness + blur_margin, half_thickness + blur_margin);
    float2 max_point = max(seg.p0, max(seg.p1, seg.p2)) + float2(half_thickness + blur_margin, half_thickness + blur_margin);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    float2 p = top_left + uv * size;

    float dist = sd_quadratic_bezier(p, seg.p0, seg.p1, seg.p2) - half_thickness;

    if (seg.stroke_thickness > 0.001) {
        dist = annular(dist, seg.stroke_thickness);
    }

    float alpha;
    if (seg.blur_radius > 0.5) {
        alpha = 1.0 / (1.0 + exp(1.7 * dist / seg.blur_radius));
    } else {
        float aa_width = 0.5 / max(transform_scale, 0.001);
        alpha = 1.0 - linearstep(-aa_width, aa_width, dist);
    }

    if (alpha < 0.001) {
        discard;
    }

    var grad: GradientGpu = reinterpret<GradientGpu>(shapes.resources[seg.gradient_index]);
    float4 color = grad.color_start;

    if (srgb) {
        color.rgb = srgb_to_linear3(color.rgb);
    }

    return float4(color.rgb, alpha * color.a);
}
