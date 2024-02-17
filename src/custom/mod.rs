use bevy::prelude::{Color, Image, Plugin, Shader, TypePath};
use bevy::asset::{Asset, Handle, load_internal_asset};
use bevy::math::Vec4;
use bevy::pbr::{Material, MaterialMeshBundle};

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, ShaderType, ShaderRef, AsBindGroupShaderType};

pub const CUSTOM_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826659);
pub const CUSTOM_FUNCTIONS_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826660);

#[derive(Default)]
pub struct CustomShaderPlugin;

impl Plugin for CustomShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(app, CUSTOM_SHADER_HANDLE, "custom_shader.wgsl", Shader::from_wgsl);
        
        load_internal_asset!(app, CUSTOM_FUNCTIONS_HANDLE, "custom_functions.wgsl", Shader::from_wgsl);

        app.add_plugins(bevy::pbr::MaterialPlugin::<CustomMaterial>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(CustomMaterialKey)]
#[uniform(0, CustomMaterialUniform)]
pub struct CustomMaterial {
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        CUSTOM_SHADER_HANDLE.into()
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
pub struct CustomMaterialKey {
    color: bool,
}

impl From<&CustomMaterial> for CustomMaterialKey {
    fn from(material: &CustomMaterial) -> Self {
        CustomMaterialKey {
            color: material.color != Color::WHITE,
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct CustomMaterialUniform {
    pub color: Vec4,
}

impl AsBindGroupShaderType<CustomMaterialUniform> for CustomMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> CustomMaterialUniform {
        CustomMaterialUniform {
            color: self.color.into(),
        }
    }
}

pub type CustomBundle = MaterialMeshBundle<CustomMaterial>;