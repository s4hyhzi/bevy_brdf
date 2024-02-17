#define_import_path custom_shader::custom_functions

#import bevy_pbr::mesh_view_bindings as view_bindings;
#import bevy_pbr::mesh_view_types;
#import bevy_pbr::shadows;
#import bevy_pbr::mesh_bindings::mesh;
#import bevy_pbr::pbr_functions;
#import bevy_pbr::lighting;
#import bevy_pbr::prepass_utils;
#import bevy_pbr::clustered_forward as clustering;
#import bevy_pbr::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT;

#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io::VertexOutput
#else
#import bevy_pbr::forward_io::VertexOutput
#endif

struct CustomInput {
    occlusion: vec3<f32>,
    frag_coord: vec4<f32>,
    world_position: vec4<f32>,
    // Normalized world normal used for shadow mapping as normal-mapping is not used for shadow
    // mapping
    world_normal: vec3<f32>,
    // Normalized normal-mapped world normal used for lighting
    N: vec3<f32>,
    // Normalized view vector in world space, pointing from the fragment world position toward the
    // view world position
    V: vec3<f32>,
    is_orthographic: bool,
    flags: u32,
};

fn pbr_input_new() -> CustomInput {
    var pbr_input: CustomInput;
    pbr_input.occlusion = vec3<f32>(1.0);

    pbr_input.frag_coord = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    pbr_input.world_position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    pbr_input.world_normal = vec3<f32>(0.0, 0.0, 1.0);

    pbr_input.is_orthographic = false;

    pbr_input.N = vec3<f32>(0.0, 0.0, 1.0);
    pbr_input.V = vec3<f32>(1.0, 0.0, 0.0);

    pbr_input.flags = 0u;

    return pbr_input;
}


fn apply_lighting(in:VertexOutput) -> vec4<f32>{
    let perceptual_roughness = 0.5;
    let reflectance = 0.5;
    let metallic = 0.0;
    let diffuse_transmission = 0.0;
    let specular_transmission = 0.0;
    let base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let roughness = 0.5;

    let n_directional_lights = view_bindings::lights.n_directional_lights;

    var custom_input:CustomInput = pbr_input_new();
    custom_input.flags = mesh[in.instance_index].flags;
    custom_input.is_orthographic = view_bindings::view.projection[3].w == 1.0;
    custom_input.V = pbr_functions::calculate_view(in.world_position, custom_input.is_orthographic);
    custom_input.frag_coord = in.position;
    custom_input.world_position = in.world_position;
    #ifdef LOAD_PREPASS_NORMALS
        custom_input.N = prepass_utils::prepass_normal(in.position, 0u);
    #else
        custom_input.N = normalize(custom_input.world_normal);
    #endif

    var output_color: vec4<f32> = base_color;

    let NdotV = max(dot(custom_input.N, custom_input.V), 0.0001);
    let R = reflect(-custom_input.V, custom_input.N);
    let F0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + output_color.rgb * metallic;
    let f_ab = lighting::F_AB(perceptual_roughness, NdotV);
    let diffuse_color = output_color.rgb * (1.0 - metallic) * (1.0 - specular_transmission) * (1.0 - diffuse_transmission);

    let view_z = dot(vec4<f32>(
        view_bindings::view.inverse_view[0].z,
        view_bindings::view.inverse_view[1].z,
        view_bindings::view.inverse_view[2].z,
        view_bindings::view.inverse_view[3].z
    ), custom_input.world_position);

    var direct_light: vec3<f32> = vec3<f32>(0.0);

    // 计算聚类索引,用于确定片元在聚类缓冲区中的位置
    let cluster_index = clustering::fragment_cluster_index(custom_input.frag_coord.xy, view_z, custom_input.is_orthographic);
    // 解包聚类索引,用于确定聚类缓冲区中的具体数据
    let offset_and_counts = clustering::unpack_offset_and_counts(cluster_index);

    for (var i: u32 = offset_and_counts[0]; i < offset_and_counts[0] + offset_and_counts[1]; i = i + 1u) {
        let light_id = clustering::get_light_id(i);
        var shadow: f32 = 1.0;
        shadow = shadows::fetch_point_shadow(light_id, custom_input.world_position, custom_input.world_normal);
        let light_contrib = lighting::point_light(custom_input.world_position.xyz, light_id, roughness, NdotV, custom_input.N, custom_input.V, R, F0, f_ab, diffuse_color);
        direct_light += light_contrib * shadow;
    }

    for (var i: u32 = offset_and_counts[0] + offset_and_counts[1]; i < offset_and_counts[0] + offset_and_counts[1] + offset_and_counts[2]; i = i + 1u) {
        let light_id = clustering::get_light_id(i);

        var shadow: f32 = 1.0;
        shadow = shadows::fetch_spot_shadow(light_id, custom_input.world_position, custom_input.world_normal);
        let light_contrib = lighting::spot_light(custom_input.world_position.xyz, light_id, roughness, NdotV, custom_input.N, custom_input.V, R, F0, f_ab, diffuse_color);
        direct_light += light_contrib * shadow;
    }

    for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
        var shadow: f32 = 1.0;
        shadow = shadows::fetch_directional_shadow(i,custom_input.world_position,custom_input.world_normal,view_z);
        var light_contrib = lighting::directional_light(i,roughness,NdotV,custom_input.N,custom_input.V,R,F0,f_ab,diffuse_color);
        
        light_contrib = shadows::cascade_debug_visualization(light_contrib, i, view_z);
        direct_light = direct_light + light_contrib * shadow;
    }

    output_color = vec4<f32>(direct_light, output_color.a);
    output_color = clustering::cluster_debug_visualization(
        output_color,
        view_z,
        custom_input.is_orthographic,
        offset_and_counts,
        cluster_index,
    );
    return output_color;
}