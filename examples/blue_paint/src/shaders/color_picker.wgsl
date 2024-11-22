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
    @location(1) pixel_uv: vec2f,
    @location(2) half_size: vec2f,
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


    return VertexOutput(clip_position, uv, pixel_uv, half_size);
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
    let c = hcl.y * 0.33; // Adjust chroma
    let l = hcl.z;

    // Convert HCL to Lab
    let lab = vec3f(
        l,
        c * cos(h),
        c * sin(h)
    );

    // Convert Lab to LMS
    var lms = vec3f(
        lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
        lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
        lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z
    );

    // Apply non-linearity
    // lms = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0/3.0));
    lms.x = pow(max(lms.x, 0.0), 3.0);
    lms.y = pow(max(lms.y, 0.0), 3.0);
    lms.z = pow(max(lms.z, 0.0), 3.0);

    // Convert LMS to RGB
    var rgb = vec3f(
          4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
        - 1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
        - 0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z
    );

    // Gamma correction and clamping
    // rgb = transfer_vec3(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)));
    // rgb = transfer_vec3(rgb);

    // Handle out-of-gamut colors
    if (any(rgb < vec3f(0.0)) || any(rgb > vec3f(1.0))) {
        rgb = vec3f(0.25, 0.25, 0.25);
    }
    // if (any(lessThan(rgb, vec3(0.0))) || any(greaterThan(rgb, vec3(1.0)))) {
    //     rgb = vec3(0.9);
    // }


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


fn square(xy: vec2<f32>) -> SquareRes {

    let size = 0.75;

    // Transform the input coordinates
    var ab = xy / (size / sqrt(2.0));
    ab = (ab + vec2<f32>(1.0)) / 2.0;

    // Check if the point is within the square bounds
    let isIn = f32(all(ab > vec2<f32>(0.0)) && all(ab < vec2<f32>(1.0)));

    // Clamp the values of ab to the square boundaries
    ab = clamp(ab, vec2<f32>(0.0), vec2<f32>(1.0));

    return SquareRes(
        isIn,
        ab,
    );
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.uv.x;
    let v = in.uv.y;

    let ring_mask = ring(in.pixel_uv, in.half_size);

    // UV coordinate and center
    let uv = in.uv * 2.0 - 1.0; // Convert to range [-1, 1]

    // Define fixed OKLCH values
    // let hue = 0.3;     // Fixed hue
    // let chroma = 0.1;  // Fixed chroma
    // let lightness = 0.1; // Fixed lightness


    let squareMask = square(uv);
    let sq_ab = squareMask.ab;



    if (ring_mask > 0.0) {
        let hcl_x = atan2(u, v) / (2.0 * PI) - 0.25;
        // need to pick magic values so that the whole wheel stays inside the rgb gamut
        
        let hcl = vec3f(hcl_x, 0.38, 0.75);

        let color = hcl2rgb(hcl);
        return vec4f(color, ring_mask);
    }

    return vec4f(0.0, 0.0, 0.0, 0.0);

    // if (squareMask.isIn > 0.0) {
    //     // dot
    //     if (distance(sq_ab, hcl.zy) < 0.02){
    //         hcl = vec3(0.0, 0.0, 1.0);
    //     }
    //     hcl = vec3(hcl.x, sq_ab.yx);
    // }

    // let grey = vec3(0.1, 0.1, 0.1);
    // let alpha = max(ring_mask, squareMask.isIn);

    // // Convert HCL to RGB and output the color
    // let color = hcl2rgb(hcl);
    // let result = mix(grey, color, alpha);

    // return vec4<f32>(color, ring_mask);
}
