struct GroupTransform {
    float2 offset;
    float scale;
    float _padding;
}

struct BoxGpu {
    float2 clip_rect_x;
    float2 clip_rect_y;
    float2 translation;
    float rotation;
    float scale;
    float2 screen_clip_x;
    float2 screen_clip_y;
    float2 scroll_offset;
    uint slab_metadata;
    float depth;
    uint group_transform_index;
    float opacity;
    uint color_override;
    uint _padding;
}

struct GlyphQuad {
    uint pos_packed;
    uint dim_packed;
    uint uv_origin_packed;
    uint color;
    uint flags_and_page;
    uint box_index;
    uint _padding;
    uint _padding2;
}

struct Params {
    float2 resolution;
    uint srgb;
    uint _padding;
}

struct Text {
    Texture2DArray<float4> mask_atlas_texture;
    Texture2DArray<float4> color_atlas_texture;
    SamplerState atlas_sampler;
    StructuredBuffer<GlyphQuad> quads;
    ConstantBuffer<Params> params;
    StructuredBuffer<BoxGpu> box_data;
    StructuredBuffer<GroupTransform> group_transforms;
}

ParameterBlock<Text> keru_text;

float2 split_i16(uint u) {
    int a = int(u & 0x0000ffffu);
    int b = int((u & 0xffff0000u) >> 16u);
    // Convert from u16 bit pattern to i16 values
    int a_i16 = (a >= 32768) ? (a - 65536) : a;
    int b_i16 = (b >= 32768) ? (b - 65536) : b;
    return float2(float(a_i16), float(b_i16));
}

float2 split(uint u) {
    return float2(
        float(u & 0x0000ffffu),
        float((u & 0xffff0000u) >> 16u)
    );
}

float4 unpack_color_u32(uint color) {
    return float4(
        float((color & 0xff000000u) >> 24u) / 255.0,
        float((color & 0x00ff0000u) >> 16u) / 255.0,
        float((color & 0x0000ff00u) >> 8u) / 255.0,
        float((color & 0x000000ffu)) / 255.0
    );
}

uint unpack_page_index(uint flags_and_page) {
    return (flags_and_page >> 24u) & 0xFFu;
}

uint get_content_type(uint flags_and_page) {
    return flags_and_page & 0x0Fu;
}

struct ClipResult {
    float2 position;
    float2 uv_offset;
    float2 dimensions;
}

ClipResult apply_clipping(
    float2 quad_pos,
    float2 quad_dim,
    float4 clip_rect,
    float2 coords
) {
    float4 quad_bounds = float4(
        quad_pos.x,
        quad_pos.y,
        quad_pos.x + quad_dim.x,
        quad_pos.y + quad_dim.y
    );

    float4 clipped_bounds = float4(
        max(quad_bounds.x, clip_rect.x),
        max(quad_bounds.y, clip_rect.y),
        max(max(quad_bounds.x, clip_rect.x), min(quad_bounds.z, clip_rect.z)),
        max(max(quad_bounds.y, clip_rect.y), min(quad_bounds.w, clip_rect.w))
    );

    float2 uv_offset = clipped_bounds.xy - quad_bounds.xy;
    float2 clipped_dim = clipped_bounds.zw - clipped_bounds.xy;

    ClipResult result;
    result.position = clipped_bounds.xy + clipped_dim * coords;
    result.uv_offset = uv_offset;
    result.dimensions = clipped_dim;
    return result;
}

void text_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv
) {
    GlyphQuad quad = keru_text.quads[instanceIndex];
    BoxGpu box = keru_text.box_data[quad.box_index];
    GroupTransform group_transform = keru_text.group_transforms[box.group_transform_index];

    float2 uv01 = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    float2 quad_pos = split_i16(quad.pos_packed);
    float2 dim = split(quad.dim_packed);

    float4 clip_rect = float4(box.clip_rect_x.x, box.clip_rect_y.x, box.clip_rect_x.y, box.clip_rect_y.y);
    ClipResult clip_result = apply_clipping(quad_pos, dim, clip_rect, uv01);

    float2 pos = clip_result.position - box.scroll_offset;
    // Scale
    float2 scaled_pos = pos * box.scale;

    // Rotate
    float cos_r = cos(box.rotation);
    float sin_r = sin(box.rotation);
    float2 rotated_pos = float2(
        cos_r * scaled_pos.x - sin_r * scaled_pos.y,
        sin_r * scaled_pos.x + cos_r * scaled_pos.y
    );

    // Translate
    float2 transformed_pos = rotated_pos + box.translation;

    // Apply group transform
    float2 group_transformed_pos = transformed_pos * group_transform.scale + group_transform.offset;

    // Convert to NDC
    float2 ndc = (group_transformed_pos / resolution) * 2.0 - 1.0;
    position = float4(ndc.x, -ndc.y, 0.0, 1.0);

    // Calculate UV coordinates for texture atlas
    float2 uv_origin = split(quad.uv_origin_packed);
    uint atlas_width, atlas_height, atlas_elements;
    keru_text.mask_atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
    float2 atlas_size = float2(float(atlas_width), float(atlas_height));
    uv = (uv_origin + clip_result.uv_offset + clip_result.dimensions * uv01) / atlas_size;
}

float4 text_fragment_pure(GlyphQuad quad, float2 uv, float4 frag_coord): SV_Target {
    // Apply screen-space clipping using built-in fragment position
    // frag_coord.xy is in pixel coordinates (screen space)
    BoxGpu box = keru_text.box_data[quad.box_index];
    if (frag_coord.x < box.screen_clip_x.x || frag_coord.x > box.screen_clip_x.y ||
        frag_coord.y < box.screen_clip_y.x || frag_coord.y > box.screen_clip_y.y) {
        discard;
    }

    uint page_index = unpack_page_index(quad.flags_and_page);
    uint content_type = get_content_type(quad.flags_and_page);

    float4 color = unpack_color_u32(quad.color);

    if (box.color_override != 0u && content_type == 0u) {
        color = unpack_color_u32(box.color_override);
    }
    color.a *= box.opacity;

    if (content_type == 1) {
        // Color bitmap text
        float4 tex_color = keru_text.color_atlas_texture.SampleLevel(
            keru_text.atlas_sampler,
            float3(uv, float(page_index)),
            0.0
        );
        return float4(color.rgb * tex_color.rgb, color.a * tex_color.a);
    } else if (content_type == 0) {
        // Mask text
        float glyph_alpha = keru_text.mask_atlas_texture.SampleLevel(
            keru_text.atlas_sampler,
            float3(uv, float(page_index)),
            0.0
        ).r;
        return float4(color.rgb, color.a * glyph_alpha);
    } else {
        // Solid rectangles for selection and cursor
        return color;
    }
}

float4 text_fragment(uint primitiveIndex, float2 uv, float4 frag_coord: SV_Position): SV_Target {
    GlyphQuad quad = keru_text.quads[primitiveIndex];
    return text_fragment_pure(quad, uv, frag_coord);
}
