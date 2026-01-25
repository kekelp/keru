// Stolen from Shadertoy.
// todo: replace this shader
struct PushConstants {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    time: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

var<push_constant> data: PushConstants;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let u = f32(vertex_index & 1u);
    let v = f32((vertex_index >> 1u) & 1u);

    out.uv = vec2<f32>(u, v);

    let x = mix(data.min_x, data.max_x, u);
    let y = mix(data.min_y, data.max_y, v);

    let ndc_x = x * 2.0 - 1.0;
    let ndc_y = 1.0 - y * 2.0;

    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    var pp = fract(p * 0.6180339887);
    pp *= 25.0;
    return fract(pp.x * pp.y * (pp.x + pp.y));
}

fn noise(x: vec2<f32>) -> f32 {
    let p = floor(x);
    let f = fract(x);
    let ff = f * f * (3.0 - 2.0 * f);
    let a = hash(p + vec2<f32>(0.0, 0.0));
    let b = hash(p + vec2<f32>(1.0, 0.0));
    let c = hash(p + vec2<f32>(0.0, 1.0));
    let d = hash(p + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, ff.x), mix(c, d, ff.x), ff.y);
}

const mtx = mat2x2<f32>(0.80, 0.60, -0.60, 0.80);

fn fbm4(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var f = 0.0;
    f += 0.5000 * (-1.0 + 2.0 * noise(p)); p = mtx * p * 2.02;
    f += 0.2500 * (-1.0 + 2.0 * noise(p)); p = mtx * p * 2.03;
    f += 0.1250 * (-1.0 + 2.0 * noise(p)); p = mtx * p * 2.01;
    f += 0.0625 * (-1.0 + 2.0 * noise(p));
    return f / 0.9375;
}

fn fbm6(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var f = 0.0;
    f += 0.500000 * noise(p); p = mtx * p * 2.02;
    f += 0.250000 * noise(p); p = mtx * p * 2.03;
    f += 0.125000 * noise(p); p = mtx * p * 2.01;
    f += 0.062500 * noise(p); p = mtx * p * 2.04;
    f += 0.031250 * noise(p); p = mtx * p * 2.01;
    f += 0.015625 * noise(p);
    return f / 0.96875;
}

fn fbm4_2(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(fbm4(p + vec2<f32>(1.0, 0.0)), fbm4(p + vec2<f32>(6.2, 0.0)));
}

fn fbm6_2(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(fbm6(p + vec2<f32>(9.2, 0.0)), fbm6(p + vec2<f32>(5.7, 0.0)));
}

struct FuncResult {
    f: f32,
    o: vec2<f32>,
    n: vec2<f32>,
}

fn func(q_in: vec2<f32>, time: f32) -> FuncResult {
    var q = q_in;
    q += 0.05 * sin(vec2<f32>(0.11, 0.13) * time + length(q) * 4.0);
    q *= 0.7 + 0.2 * cos(0.05 * time);
    var o = 0.5 + 0.5 * fbm4_2(q);
    o += 0.02 * sin(vec2<f32>(0.13, 0.11) * time * length(o));
    let n = fbm6_2(4.0 * o);
    let p = q + 2.0 * n + 1.0;
    var f = 0.5 + 0.5 * fbm4(2.0 * p);
    f = mix(f, f * f * f * 3.5, f * abs(n.x));
    f *= 1.0 - 0.5 * pow(0.5 + 0.5 * sin(8.0 * p.x) * sin(8.0 * p.y), 8.0);
    return FuncResult(f, o, n);
}

fn funcs(q: vec2<f32>, time: f32) -> f32 {
    return func(q, time).f;
}

@fragment
fn fs_main(vert_out: VertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(data.max_x - data.min_x, data.max_y - data.min_y);
    let uv = vert_out.uv * resolution;
    let time = data.time * 15.0;

    let q = (2.0 * uv - resolution) / resolution.y;

    let result = func(q, time);
    let f = result.f;
    let o = result.o;
    let n = result.n;

    var col = vec3<f32>(0.2, 0.1, 0.4);
    col = mix(col, vec3<f32>(0.3, 0.05, 0.05), f);
    col = mix(col, vec3<f32>(0.9, 0.9, 0.9), dot(n, n));
    col = mix(col, vec3<f32>(0.5, 0.2, 0.2), 0.5 * o.y * o.y);
    col = mix(col, vec3<f32>(0.0, 0.2, 0.4), 0.5 * smoothstep(1.2, 1.3, abs(n.y) + abs(n.x)));
    col *= f * 2.0;

    let ex = vec2<f32>(1.0 / resolution.x, 0.0);
    let ey = vec2<f32>(0.0, 1.0 / resolution.y);
    let nor = normalize(vec3<f32>(funcs(q + ex, time) - f, ex.x, funcs(q + ey, time) - f));

    let lig = normalize(vec3<f32>(0.9, -0.2, -0.4));
    let dif = clamp(0.3 + 0.7 * dot(nor, lig), 0.0, 1.0);

    var lin = vec3<f32>(0.85, 0.90, 0.95) * (nor.y * 0.5 + 0.5);
    lin += vec3<f32>(0.15, 0.10, 0.05) * dif;

    col *= lin;
    col = vec3<f32>(1.0, 1.0, 1.0) - col;
    col = col * col;
    col *= vec3<f32>(1.2, 1.25, 1.2);

    let p = uv / resolution;
    col *= 0.5 + 0.5 * sqrt(16.0 * p.x * p.y * (1.0 - p.x) * (1.0 - p.y));

    return vec4<f32>(col, 1.0);
}
