#define_import_path bevy_brdf::toon_fragment

#import bevy_pbr::pbr_fragment::pbr_input_from_vertex_output;

#import bevy_brdf::toon_bindings;

#import bevy_pbr::{
    pbr_functions,
    pbr_types,
    mesh_bindings::mesh,
    mesh_view_bindings::view,
    parallax_mapping::parallaxed_uv,
}


#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::VertexOutput
#else
#import bevy_pbr::forward_io::VertexOutput
#endif

fn toon_input_from_standard_material(
    in: VertexOutput,
    is_front: bool,
) -> pbr_types::PbrInput {
    let double_sided = (toon_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    var pbr_input: pbr_types::PbrInput = pbr_input_from_vertex_output(in, is_front, double_sided);

    return pbr_input;
}