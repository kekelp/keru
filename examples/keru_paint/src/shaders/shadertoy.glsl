const float PI = 3.14159265358979323846;

float transfer(float v) {
    return v <= 0.0031308 ? 12.92 * v : 1.055 *pow(v, 0.4166666666666667) - 0.055;
}

vec3 transfer(vec3 v) {
    return vec3(transfer(v.x), transfer(v.y), transfer(v.z));
}

vec3 hcl2rgb(vec3 hcl) {
    hcl.y *= 0.33;
    
    vec3 lab = vec3(
        hcl.z,
        hcl.y * cos(hcl.x * PI*2.0),
        hcl.y * sin(hcl.x * PI*2.0)
    );
    
    vec3 lms = vec3(
        lab.x + 0.3963377774f * lab.y + 0.2158037573f * lab.z,
        lab.x - 0.1055613458f * lab.y - 0.0638541728f * lab.z,
        lab.x - 0.0894841775f * lab.y - 1.2914855480f * lab.z
    );
    
    lms = pow(max(lms, vec3(0.0)), vec3(3.0));
    
    vec3 rgb = vec3(
        +4.0767416621f * lms.x - 3.3077115913f * lms.y + 0.2309699292f * lms.z,
        -1.2684380046f * lms.x + 2.6097574011f * lms.y - 0.3413193965f * lms.z,
        -0.0041960863f * lms.x - 0.7034186147f * lms.y + 1.7076147010f * lms.z
    );
     
    rgb = transfer(rgb);
    
    if (any(lessThan(rgb, vec3(0.0))) || any(greaterThan(rgb, vec3(1.0)))) {
        rgb = vec3(0.5);
    }

    return rgb;
}

vec2 uv2xy(vec2 uv) {
    return ((uv * vec2(iResolution.x / iResolution.y, 1.0)) * 2.0 - 1.0);
}

bool ring(vec2 xy, out float t) {
    t = atan(xy.y, xy.x) / (PI * 2.0) - 0.25;
    float r = length(xy);
    return r > 0.8 && r < 0.9;
}

bool square(vec2 xy, out vec2 ab) {
    ab = xy;
    ab = ab / (0.7 / sqrt(2.0));
    ab = (ab + 1.0) / 2.0;
    bool isIn = all(greaterThan(ab, vec2(0.0))) && all(lessThan(ab, vec2(1.0)));
    ab = clamp(ab, vec2(0.0), vec2(1.0));
    return isIn;
}

vec4 changeState(vec4 state) {
    bool grabbedRing = state.w == 1.0;
    bool grabbedSquare = state.w == 2.0;
    
    vec2 mouseXY = uv2xy((iMouse.xy + vec2(0.5)) / iResolution.xy);
    bool clicked = iMouse.w > 0.0;

    float mouseT;
    bool mouseInRing = ring(mouseXY, mouseT);
    
    vec2 mouseAB;
    bool mouseInSquare = square(mouseXY, mouseAB);
    
    if (clicked) {
        grabbedRing = mouseInRing;
        grabbedSquare = mouseInSquare;
    }
    
    if (grabbedRing) {
        state.x = mouseT;
    }

    if (grabbedSquare) {
        state.zy = mouseAB;
    }
    
    state.w = 0.0;
    if (grabbedRing) state.w = 1.0;
    if (grabbedSquare) state.w = 2.0;
    
    return state;
}

vec3 getHCL(vec4 state, vec2 fragCoord) {
    vec2 fragXY = uv2xy(fragCoord / iResolution.xy);
 
    vec3 hcl = state.xyz;
    
    float fragT;
    if (ring(fragXY, fragT)) {
        if (abs(fragT - hcl.x) < 0.002) {
            return vec3(0.0, 0.0, 1.0);
        }
        return vec3(fragT, 0.38, 0.75);
        //bool isReachable = hcl2rgb(vec3(fragT, state.yz)) != vec3(0.5);
        //return vec3(fragT, 0.33, isReachable ? 0.78 : 0.65);
    }

    vec2 fragAB;
    if (square(fragXY, fragAB)) {
        if (distance(fragAB, hcl.zy) < 0.02){
            return vec3(0.0, 0.0, 1.0);
        }
        return vec3(hcl.x, fragAB.yx);
    }

    if (fragXY.x > 1.0) {
        return hcl;
    }

    return vec3(0.0, 0.0, 0.5);
}

void mainImage(out vec4 fragColor, vec2 fragCoord) {
    vec4 state = texture(iChannel0, vec2(0.5) / iResolution.xy);
    
    if (ivec2(fragCoord) == ivec2(0)) {
        fragColor = changeState(state);
    } else {
        fragColor.rgb = hcl2rgb(getHCL(state, fragCoord));
    }
}
