const PI: f32 = 3.141592653589793;
const color = vec3<f32>(0.2, 0.3, 0.8);

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let clip_position = vec4<f32>(in.pos, 0.0, 1.0);
    
    let i_x = f32(in.index % 2);
    let i_y = f32(in.index >= 2);

    let uv = vec2f(i_x, i_y);

    return VertexOutput(clip_position, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.uv.x, in.uv.y, 0.0, 1.0);
    // return vec4<f32>(in.uv.x, in.uv.y, 0.0, 1.0);
}
