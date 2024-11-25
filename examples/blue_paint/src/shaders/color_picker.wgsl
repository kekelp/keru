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
    @builtin(instance_index) instance_index: u32,
    @location(0) xs: vec2<f32>,
    @location(1) ys: vec2<f32>,
    @location(2) z: f32,
    @location(3) hcl_color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2f,
    @location(1) pixel_uv: vec2f,
    @location(2) half_size: vec2f,
    @location(3) instance_index: u32,
    @location(4) hcl_color: vec3<f32>,
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

    // get the corners' coordinates in reasonable units.
    // -L/2 <-- 0 --> +L/2
    // where L = length of the rect side in real pixels
    let half_size = vec2f( 
        (in.xs[1] - in.xs[0]) * base_unif.window_size.x / 2.0, 
        (in.ys[1] - in.ys[0]) * base_unif.window_size.y / 2.0, 
    );
    let pixel_uv = (2.0 * vec2f(vec2u(i_x, i_y)) - 1.0) * half_size;    

    let u = f32(i_x) * 2.0 - 1.0;
    let v = f32(i_y) * 2.0 - 1.0;
    let uv = vec2f(u, v);

    return VertexOutput(clip_position, uv, pixel_uv, half_size, in.instance_index, in.hcl_color);
}

// Transfer function for gamma correction
fn transfer(v: f32) -> f32 {
    return select(12.92 * v, 1.055 * pow(v, 1.0 / 2.4) - 0.055, v > 0.0031308);
}

fn transfer_vec3(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(transfer(v.x), transfer(v.y), transfer(v.z));
}

// Convert OKLCH to RGB
fn hcl2rgb(hcl: vec3<f32>) -> vec3<f32> {
    let h = hcl.x * 2.0 * PI;
    let c = hcl.y * 0.33;
    let l = hcl.z;

    let lab = vec3f(
        l,
        c * cos(h),
        c * sin(h)
    );

    var lms = vec3f(
        lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
        lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
        lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
    );

    lms.x = pow(max(lms.x, 0.0), 3.0);
    lms.y = pow(max(lms.y, 0.0), 3.0);
    lms.z = pow(max(lms.z, 0.0), 3.0);

    // Convert LMS to RGB
    var rgb = vec3f(
          4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
        - 1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
        - 0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
    );

    // Handle out-of-gamut colors
    // todo: antialiasing
    if (any(rgb < vec3f(0.0)) || any(rgb > vec3f(1.0))) {
        rgb = vec3f(0.17, 0.17, 0.17);
    }

    return rgb;
}

// Antialiased ring
fn ring(pixel_uv: vec2<f32>, half_size: vec2<f32>) -> f32 {
    const WIDTH: f32 = 60.0; // pixels
    
    let outer_radius = half_size.x;
    let inner_radius = half_size.x - WIDTH;
    let smoothness = 1.0;

    let r = length(pixel_uv);
    let inner = smoothstep(inner_radius - smoothness, inner_radius + smoothness, r);
    let outer = 1.0 - smoothstep(outer_radius - smoothness, outer_radius + smoothness, r);
    return inner * outer;
}

struct SquareRes {
    isIn: f32,
    ab: vec2<f32>,
} 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.uv.x;
    let v = in.uv.y;

    // hue wheel
    if (in.instance_index == 0) {
        // convert to range [-1, 1]
        let uv = in.uv * 2.0 - 1.0;

        let ring_mask = ring(in.pixel_uv, in.half_size);

        if (ring_mask > 0.0) {
            let hcl_hue = atan2(u, -v) / (2.0 * PI);
            
            // need to pick magic values so that the whole wheel stays inside the rgb gamut
            let hcl = vec3f(hcl_hue, 0.38, 0.75);

            let color = hcl2rgb(hcl);
            return vec4f(color, ring_mask);
        }

        return vec4f(0.0, 0.0, 0.0, 0.0);

    }
    // main square
    else if (in.instance_index == 1) {
        // convert back to range [0, 1] ...
        let uv = (in.uv + 1.0) / 2.0;

        let hue = in.hcl_color.x;
        let hcl_x = hue / (2.0 * PI);
        let hcl = vec3(hcl_x, uv.yx);

        let color = hcl2rgb(hcl);

        return vec4<f32>(color, 1.0);
    }

    return vec4f(0.0, 1.0, 0.0, 0.8);
}
