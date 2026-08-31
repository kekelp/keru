import keru_text;

struct VertexOutput {
    nointerpolation GlyphQuad quad;
    float4 position: SV_Position;
    float2 uv;
}

[shader("vertex")]
 VertexOutput vertexMain(
    GlyphQuad input,
    uint vertexIndex: SV_VulkanVertexID,
    uint instanceIndex: SV_VulkanInstanceID,
) {
    VertexOutput output;
    output.quad = input;

    text_vertex(
        instanceIndex,
        vertexIndex,
        keru_text.params.resolution,
        output.position,
        output.uv
    );

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {
    return text_fragment_pure(input.quad, input.uv, input.position);
}
