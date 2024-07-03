struct BaseUniforms {
    @location(1) screen_size: vec2f,
    @location(0) t: f32,
};

@group(0) @binding(0)
var<uniform> base_unif: BaseUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(1)
var my_texture: texture_2d<f32>;
@group(0) @binding(2)
var my_sampler: sampler;

struct Uniforms {
    @location(0) scale: vec2f,
    @location(1) cos: f32,
    @location(2) sin: f32,
    @location(3) translation: vec2f,
    @location(4) image_size: vec2f,
};
@group(0) @binding(3)
var<uniform> unif: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {

    // let h_times_aspect = unif.image_size.y / base_unif.screen_size.y * (base_unif.screen_size.x / base_unif.screen_size.y);
    let w =              unif.image_size.x / base_unif.screen_size.x;
    let h_times_aspect = unif.image_size.y / base_unif.screen_size.x;


    var positions = array<vec4<f32>, 6>(
        vec4<f32>(-w, -h_times_aspect, 0.0, 1.0),
        vec4<f32>( w, -h_times_aspect, 0.0, 1.0),
        vec4<f32>(-w,  h_times_aspect, 0.0, 1.0),
        vec4<f32>( w, -h_times_aspect, 0.0, 1.0),
        vec4<f32>( w,  h_times_aspect, 0.0, 1.0),
        vec4<f32>(-w,  h_times_aspect, 0.0, 1.0),
    );
    
    var tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    var output: VertexOutput;
    
    let aspect = base_unif.screen_size.y / base_unif.screen_size.x;

    output.position = positions[vertex_index];
    output.position.x = output.position.x * unif.scale.x;
    output.position.y = output.position.y * unif.scale.y;


    let originalX = output.position.x;
    let originalY = output.position.y;

    output.position.x = originalX * unif.cos - originalY * unif.sin; 
    output.position.y = originalX * unif.sin + originalY * unif.cos;


    output.position.x = output.position.x + unif.translation.x;
    output.position.y = output.position.y - (unif.translation.y) * aspect;

    output.position.y = output.position.y / aspect;

    output.tex_coords = tex_coords[vertex_index];
    return output;
}

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    // return vec4(1.0, 0.0, base_unif.t, 1.0);
    return textureSample(my_texture, my_sampler, tex_coords);
}
