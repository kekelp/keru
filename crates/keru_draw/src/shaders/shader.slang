// The order of the imports has to match the binding indices in the Rust code!
// If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
import keru_draw;
import keru_text;

enum PrimitiveType: uint {
    Box = 0,
    Circle = 1,
    Segment = 2,
    Text = 3,
    Grid = 4,
    Triangle = 5,
    Hexagon = 6,
    QuadraticBezier = 7,
}

struct VertexOutput {
    float4 position: SV_Position;
    nointerpolation uint primitiveType: PRIMTYPE;
    nointerpolation uint primitiveIndex: PRIMINDEX;
    nointerpolation uint clipRectIndex: CLIPRECTINDEX;
    float2 uv: TEXCOORD0;
    float2 screen_pos: TEXCOORD1;
    float transform_scale: TEXCOORD2;
}

struct VertexInput {
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
    uint transformIndex: TRANSFORMINDEX;
    uint clipRectIndex: CLIPRECTINDEX;
}


[shader("vertex")]
VertexOutput vertexMain(
    VertexInput input,
    uint vertexIndex: SV_VulkanVertexID
) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;
    output.primitiveIndex = input.primitiveIndex;
    output.clipRectIndex = input.clipRectIndex;
    output.transform_scale = 1.0; // Default scale

    // grab the screen resolution from keru_text
    float2 resolution = keru_text.params.resolution;

    if (input.primitiveType == PrimitiveType::Box) {
        box_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Circle) {
        circle_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Segment) {
        segment_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Grid) {
        grid_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Triangle) {
        triangle_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Hexagon) {
        hexagon_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::QuadraticBezier) {
        quadratic_bezier_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv, output.screen_pos);
    }
    else if (input.primitiveType == PrimitiveType::Text) {
        text_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
        output.screen_pos = float2(0.0, 0.0); // Text doesn't use screen_pos for clipping
    }

    // Apply transform
    // keru_text quads are already aware of their transform
    if (input.primitiveType != PrimitiveType::Text) {
        // The position is currently in NDC, so convert back to screen space
        float2 screen_pos = (output.position.xy * float2(1.0, -1.0) + 1.0) * resolution * 0.5;

        // Apply the transform
        Transform transform = reinterpret<Transform>(shapes.resources[input.transformIndex].a);
        float2 transformed_pos = screen_pos * transform.scale + transform.offset;

        // Store the scale factor for antialiasing
        output.transform_scale = transform.scale;

        // Store the transformed screen position for clipping in fragment shader
        output.screen_pos = transformed_pos;

        // Convert back to NDC
        float2 ndc = (transformed_pos / resolution) * 2.0 - 1.0;
        output.position = float4(ndc.x, -ndc.y, output.position.z, 1.0);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {

    // grab the srgb flag from keru_text
    bool srgb = keru_text.params.srgb != 0;

    if (input.primitiveType == PrimitiveType::Box) {
        return box_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Circle) {
        return circle_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Segment) {
        return segment_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Grid) {
        return grid_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Triangle) {
        return triangle_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Hexagon) {
        return hexagon_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::QuadraticBezier) {
        return quadratic_bezier_fragment(input.primitiveIndex, input.clipRectIndex, input.uv, input.screen_pos, srgb, input.transform_scale);
    }
    else if (input.primitiveType == PrimitiveType::Text) {
        return text_fragment(input.primitiveIndex, input.uv, input.position);
    }

    return float4(0.0, 1.0, 0.0, 1.0);  // Greenscreen for error
}
