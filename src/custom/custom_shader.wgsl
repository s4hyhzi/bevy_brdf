#import bevy_pbr::forward_io::VertexOutput

#import custom_shader::custom_functions::apply_lighting;

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: CustomMaterial;
@group(1) @binding(1) var base_color_texture: texture_2d<f32>;
@group(1) @binding(2) var base_color_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    var out: vec4<f32>;
    let light_color = apply_lighting(in);
    #ifdef USE_COLOR
    out = material.color * light_color;
    #else
    out = vec4(1.0) * light_color;
    #endif
    return out;
}
