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
    @location(0) xs: vec2<f32>,
    @location(1) ys: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2f,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // triangle strip indexes
    let i_x = u32(in.index % 2);
    let i_y = u32(in.index >= 2);

    let x = in.xs[i_x];
    let y = in.ys[i_y];
    let clip_position = vec4f(x, y, 0.0, 1.0);

    let rect = vec2f(in.xs[1] - in.xs[0], in.ys[1] - in.ys[0]);

    let width = rect.x * base_unif.window_size.x;
    let height = rect.y * base_unif.window_size.y;
    let aspect = width / height;

    // let u = f32(i_x) * aspect - (((width - height)) / base_unif.window_size.x );
    let u = f32(i_x);
    let v = f32(i_y);

    let uv = vec2f(u, v);

    return VertexOutput(clip_position, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var x = in.uv.x;
    if x < 0.0 || x > 1.0 {
        return vec4f(0.5, 0.5, 0.5, 1.0);
    }

    var y = in.uv.y;

    return vec4f(x, 0.0, y, 1.0);
}
