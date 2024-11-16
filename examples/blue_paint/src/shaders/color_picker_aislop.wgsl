const PI: f32 = 3.141592653589793;

fn transfer(v: f32) -> f32 {
    if (v <= 0.0031308) {
        return 12.92 * v;
    }
    return 1.055 * pow(v, 0.4166666666666667) - 0.055;
}

fn transfer_vec3(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(transfer(v.x), transfer(v.y), transfer(v.z));
}

fn hcl2rgb(hcl: vec3<f32>) -> vec3<f32> {
    var adjusted_hcl = hcl;
    adjusted_hcl.y *= 0.33;

    let lab = vec3<f32>(
        adjusted_hcl.z,
        adjusted_hcl.y * cos(adjusted_hcl.x * PI * 2.0),
        adjusted_hcl.y * sin(adjusted_hcl.x * PI * 2.0)
    );

    let lms = vec3<f32>(
        lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
        lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
        lab.x - 0.0894841775 * lab.y - 1.291485548 * lab.z
    );

    var lms_adjusted = max(lms, vec3<f32>(0.0));
    lms_adjusted = pow(lms_adjusted, vec3<f32>(3.0));

    var rgb = vec3<f32>(
        4.0767416621 * lms_adjusted.x - 3.3077115913 * lms_adjusted.y + 0.2309699292 * lms_adjusted.z,
        -1.2684380046 * lms_adjusted.x + 2.6097574011 * lms_adjusted.y - 0.3413193965 * lms_adjusted.z,
        -0.0041960863 * lms_adjusted.x - 0.7034186147 * lms_adjusted.y + 1.707614701 * lms_adjusted.z
    );

    rgb = transfer_vec3(rgb);

    if (any(rgb < vec3<f32>(0.0)) || any(rgb > vec3<f32>(1.0))) {
        rgb = vec3<f32>(0.5);
    }

    return rgb;
}

fn uv2xy(uv: vec2<f32>, resolution: vec2<f32>) -> vec2<f32> {
    return (uv * vec2<f32>(resolution.x / resolution.y, 1.0)) * 2.0 - vec2<f32>(1.0);
}

fn ring(xy: vec2<f32>, out_t: ptr<function, f32>) -> bool {
    let r = length(xy);
    let t = atan2(xy.y, xy.x) / (PI * 2.0) - 0.25;
    *out_t = t;
    return r > 0.8 && r < 0.9;
}

fn square(xy: vec2<f32>, out_ab: ptr<function, vec2<f32>>) -> bool {
    var ab = xy / (0.7 / sqrt(2.0));
    ab = (ab + vec2<f32>(1.0)) / 2.0;
    let isIn = all(ab > vec2<f32>(0.0)) && all(ab < vec2<f32>(1.0));
    *out_ab = clamp(ab, vec2<f32>(0.0), vec2<f32>(1.0));
    return isIn;
}

fn changeState(
    state: vec4<f32>, 
    mouse: vec4<f32>, 
    resolution: vec2<f32>
) -> vec4<f32> {
    var modified_state = state;

    let mouseXY = uv2xy((mouse.xy + vec2<f32>(0.5)) / resolution, resolution);
    let clicked = mouse.w > 0.0;

    var mouseT: f32 = 0.0;
    let mouseInRing = ring(mouseXY, &mouseT);

    var mouseAB = vec2<f32>(0.0);
    let mouseInSquare = square(mouseXY, &mouseAB);

    if (clicked) {
        if (mouseInRing) {
            modified_state.w = 1.0;
        } else if (mouseInSquare) {
            modified_state.w = 2.0;
        } else {
            modified_state.w = 0.0;
        }
    }

    if (modified_state.w == 1.0) {
        modified_state.x = mouseT;
    } else if (modified_state.w == 2.0) {
        modified_state.z = mouseAB.x;
        modified_state.y = mouseAB.y;
    }

    return modified_state;
}

fn getHCL(
    state: vec4<f32>, 
    fragCoord: vec2<f32>, 
    resolution: vec2<f32>
) -> vec3<f32> {
    let fragXY = uv2xy(fragCoord / resolution, resolution);

    let hcl = state.xyz;

    var fragT: f32 = 0.0;
    if (ring(fragXY, &fragT)) {
        if (abs(fragT - hcl.x) < 0.002) {
            return vec3<f32>(0.0, 0.0, 1.0);
        }
        return vec3<f32>(fragT, 0.38, 0.75);
    }

    var fragAB = vec2<f32>(0.0);
    if (square(fragXY, &fragAB)) {
        if (distance(fragAB, hcl.zy) < 0.02) {
            return vec3<f32>(0.0, 0.0, 1.0);
        }
        return vec3<f32>(hcl.x, fragAB.yx);
    }

    if (fragXY.x > 1.0) {
        return hcl;
    }

    return vec3<f32>(0.0, 0.0, 0.5);
}

@binding(0) var<uniform> iResolution: vec2<f32>;
@group(0) @binding(1) var<uniform> iMouse: vec4<f32>;
@group(0) @binding(2) var iChannel0: texture_2d<f32>;

@fragment
fn mainImage(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let state = textureLoad(iChannel0, vec2<i32>(0, 0), 0);

    if (fragCoord.xy == vec2<f32>(0.0)) {
        return vec4<f32>(changeState(state, iMouse, iResolution), 1.0);
    } else {
        return vec4<f32>(hcl2rgb(getHCL(state, fragCoord.xy, iResolution)), 1.0);
    }
}

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}