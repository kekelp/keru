void quad_vertex(
    float2 top_left,
    float2 size,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
) {
    float2 uv01 = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    // Calculate quad position
    float2 p = top_left + uv01 * size;

    // Convert to NDC
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y
    uv = uv01;
}
