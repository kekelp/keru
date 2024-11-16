const PI: f32 = 3.141592653589793;
const color = vec3<f32>(0.2, 0.3, 0.8);

struct BaseUniforms {
    @location(1) window_size: vec2f,
    @location(0) t: f32,
};

@group(0) @binding(0)
var<uniform> base_unif: BaseUniforms;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec2f
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2f,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let clip_position = vec4<f32>(in.pos, 0.0, 1.0);
    
    let aspect_ratio = base_unif.window_size.x / base_unif.window_size.y;

    let i_x = f32(in.index % 2);
    let i_y = f32(in.index >= 2);

    let uv = vec2f(i_x * aspect_ratio, i_y);

    return VertexOutput(clip_position, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.uv.x, 0.0, in.uv.y, 1.0);
}
