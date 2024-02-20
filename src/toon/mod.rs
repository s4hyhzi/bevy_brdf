use bevy::asset::{load_internal_asset, Asset, AssetApp, Assets, Handle};
use bevy::math::Vec4;
use bevy::pbr::deferred::DEFAULT_PBR_DEFERRED_LIGHTING_PASS_ID;
use bevy::pbr::{AlphaMode, Material, MaterialMeshBundle};
use bevy::prelude::{Color, Image, Plugin, Shader};

use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType};

const ALPHA_MODE_MASK_BITS: u32 = 0b111;
const ALPHA_MODE_SHIFT_BITS: u32 = 32 - ALPHA_MODE_MASK_BITS.count_ones();
const ALPHA_MODE_OPAQUE: u32 = 0 << ALPHA_MODE_SHIFT_BITS; // ← Values are just sequential values bitshifted into
const ALPHA_MODE_MASK: u32 = 1 << ALPHA_MODE_SHIFT_BITS; //   the bitmask, and can range from 0 to 7.
const ALPHA_MODE_BLEND: u32 = 2 << ALPHA_MODE_SHIFT_BITS; //
const ALPHA_MODE_PREMULTIPLIED: u32 = 3 << ALPHA_MODE_SHIFT_BITS; //
const ALPHA_MODE_ADD: u32 = 4 << ALPHA_MODE_SHIFT_BITS; //   Right now only values 0–5 are used, which still gives
const ALPHA_MODE_MULTIPLY: u32 = 5 << ALPHA_MODE_SHIFT_BITS; // ← us "room" for two more modes without adding more bits
const NONE: u32 = 0;

pub const TOON_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826659);
pub const TOON_BINDDINGS_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826660);
pub const TOON_FRAGMENT_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826661);

#[derive(Default)]
pub struct ToonShaderPlugin;

impl Plugin for ToonShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            TOON_SHADER_HANDLE,
            "toon_shader.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TOON_BINDDINGS_HANDLE,
            "toon_bindings.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TOON_FRAGMENT_HANDLE,
            "toon_fragment.wgsl",
            Shader::from_wgsl
        );

        app.register_asset_reflect::<ToonMaterial>()
            .add_plugins(bevy::pbr::MaterialPlugin::<ToonMaterial>::default());

        app.world
            .resource_mut::<Assets<ToonMaterial>>()
            .insert(Handle::<ToonMaterial>::default(), ToonMaterial::default())
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[bind_group_data(ToonMaterialKey)]
#[uniform(0, ToonMaterialUniform)]
#[reflect(Default, Debug)]
pub struct ToonMaterial {
    pub base_color: Color,
    pub alpha_mode: AlphaMode,
    pub deferred_lighting_pass_id: u8,
}

impl Default for ToonMaterial {
    fn default() -> Self {
        ToonMaterial {
            // White because it gets multiplied with texture values if someone uses
            // a texture.
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Opaque,
            deferred_lighting_pass_id: DEFAULT_PBR_DEFERRED_LIGHTING_PASS_ID,
        }
    }
}

impl From<Color> for ToonMaterial {
    fn from(color: Color) -> Self {
        ToonMaterial {
            base_color: color,
            alpha_mode: if color.a() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..Default::default()
        }
    }
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        TOON_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            let shader_defs = &mut fragment.shader_defs;

            if key.bind_group_data.color {
                shader_defs.push("USE_COLOR".into());
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ToonMaterialKey {
    color: bool,
}

impl From<&ToonMaterial> for ToonMaterialKey {
    fn from(material: &ToonMaterial) -> Self {
        ToonMaterialKey {
            color: material.base_color != Color::WHITE,
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct ToonMaterialUniform {
    pub base_color: Vec4,
    pub flags: u32,
    pub alpha_cutoff: f32,
    pub deferred_lighting_pass_id: u32,
}

impl AsBindGroupShaderType<ToonMaterialUniform> for ToonMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ToonMaterialUniform {
        let mut flags = NONE;

        let mut alpha_cutoff = 0.5;
        match self.alpha_mode {
            AlphaMode::Opaque => flags |= ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => flags |= ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => flags |= ALPHA_MODE_ADD,
            AlphaMode::Multiply => flags |= ALPHA_MODE_MULTIPLY,
        };

        ToonMaterialUniform {
            base_color: self.base_color.as_linear_rgba_f32().into(),
            flags,
            alpha_cutoff,
            deferred_lighting_pass_id: self.deferred_lighting_pass_id as u32,
        }
    }
}

pub type ToonBundle = MaterialMeshBundle<ToonMaterial>;
