#import bevy_pbr::forward_io::VertexOutput

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: CustomMaterial;
@group(1) @binding(1) var base_color_texture: texture_2d<f32>;
@group(1) @binding(2) var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var out: vec4<f32>;
    #ifdef USE_COLOR
    out = material.color;
    #else
    out = vec4(1.0);
    #endif
    return out;
}
