#import bevy_pbr::{
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
}

#import bevy_brdf::toon_fragment::toon_input_from_standard_material;

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = toon_input_from_standard_material(in, is_front);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    let colors: array<vec4<f32>, 2> = array<vec4<f32>, 2>(
        vec4<f32>(0.399287, 0.524083, 0.699776, 1.0),
        vec4<f32>(0.739922, 0.787485, 0.846145, 1.0)
    );

    // 假设 out.color 是一个 vec4<f32> 类型，并且已经定义好了
    var t: f32 = (out.color.r - 0.109155) / (0.1637 - 0.109155);
    out.color = mix(colors[0], colors[1], clamp(t, 0.0, 1.0));


    return out;
}
