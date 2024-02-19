#define_import_path bevy_brdf::toon_bindings

struct ToonMaterial {
    base_color: vec4<f32>,
    flags: u32,
    alpha_cutoff: f32,
    deferred_lighting_pass_id: u32,
};

@group(1) @binding(0) var<uniform> material: ToonMaterial;