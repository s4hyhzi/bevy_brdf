use bevy::asset::{load_internal_asset, Asset, Handle};
use bevy::math::Vec4;
use bevy::pbr::deferred::DEFAULT_PBR_DEFERRED_LIGHTING_PASS_ID;
use bevy::pbr::{
    AlphaMode, Material, MaterialMeshBundle, OpaqueRendererMethod, ParallaxMappingMethod,
};
use bevy::prelude::{Color, Image, Plugin, Shader};

use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupShaderType, Face, ShaderRef, ShaderType, TextureFormat,
};

const ALPHA_MODE_MASK_BITS: u32 = 0b111;
const ALPHA_MODE_SHIFT_BITS: u32 = 32 - ALPHA_MODE_MASK_BITS.count_ones();
const BASE_COLOR_TEXTURE: u32 = 1 << 0;
const EMISSIVE_TEXTURE: u32 = 1 << 1;
const METALLIC_ROUGHNESS_TEXTURE: u32 = 1 << 2;
const OCCLUSION_TEXTURE: u32 = 1 << 3;
const DOUBLE_SIDED: u32 = 1 << 4;
const UNLIT: u32 = 1 << 5;
const TWO_COMPONENT_NORMAL_MAP: u32 = 1 << 6;
const FLIP_NORMAL_MAP_Y: u32 = 1 << 7;
const FOG_ENABLED: u32 = 1 << 8;
const DEPTH_MAP: u32 = 1 << 9; // Used for parallax mapping
const SPECULAR_TRANSMISSION_TEXTURE: u32 = 1 << 10;
const THICKNESS_TEXTURE: u32 = 1 << 11;
const DIFFUSE_TRANSMISSION_TEXTURE: u32 = 1 << 12;
const ATTENUATION_ENABLED: u32 = 1 << 13;
const ALPHA_MODE_RESERVED_BITS: u32 = ALPHA_MODE_MASK_BITS << ALPHA_MODE_SHIFT_BITS; // ← Bitmask reserving bits for the `AlphaMode`
const ALPHA_MODE_OPAQUE: u32 = 0 << ALPHA_MODE_SHIFT_BITS; // ← Values are just sequential values bitshifted into
const ALPHA_MODE_MASK: u32 = 1 << ALPHA_MODE_SHIFT_BITS; //   the bitmask, and can range from 0 to 7.
const ALPHA_MODE_BLEND: u32 = 2 << ALPHA_MODE_SHIFT_BITS; //
const ALPHA_MODE_PREMULTIPLIED: u32 = 3 << ALPHA_MODE_SHIFT_BITS; //
const ALPHA_MODE_ADD: u32 = 4 << ALPHA_MODE_SHIFT_BITS; //   Right now only values 0–5 are used, which still gives
const ALPHA_MODE_MULTIPLY: u32 = 5 << ALPHA_MODE_SHIFT_BITS; // ← us "room" for two more modes without adding more bits
const NONE: u32 = 0;
const UNINITIALIZED: u32 = 0xFFFF;

pub const CUSTOM_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(1107985723454826659);

#[derive(Default)]
pub struct CustomShaderPlugin;

impl Plugin for CustomShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            CUSTOM_SHADER_HANDLE,
            "custom_shader.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(bevy::pbr::MaterialPlugin::<CustomMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[bind_group_data(CustomMaterialKey)]
#[uniform(0, CustomMaterialUniform)]
#[reflect(Default, Debug)]
pub struct CustomMaterial {
    pub base_color: Color,
    #[texture(1)]
    #[sampler(2)]
    #[dependency]
    pub base_color_texture: Option<Handle<Image>>,
    pub emissive: Color,
    #[texture(3)]
    #[sampler(4)]
    #[dependency]
    pub emissive_texture: Option<Handle<Image>>,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    #[texture(5)]
    #[sampler(6)]
    #[dependency]
    pub metallic_roughness_texture: Option<Handle<Image>>,
    #[doc(alias = "specular_intensity")]
    pub reflectance: f32,
    #[doc(alias = "translucency")]
    pub diffuse_transmission: f32,
    #[texture(17)]
    #[sampler(18)]
    #[cfg(feature = "pbr_transmission_textures")]
    pub diffuse_transmission_texture: Option<Handle<Image>>,
    #[doc(alias = "refraction")]
    pub specular_transmission: f32,
    #[texture(13)]
    #[sampler(14)]
    #[cfg(feature = "pbr_transmission_textures")]
    pub specular_transmission_texture: Option<Handle<Image>>,
    #[doc(alias = "volume")]
    #[doc(alias = "thin_walled")]
    pub thickness: f32,
    #[texture(15)]
    #[sampler(16)]
    #[cfg(feature = "pbr_transmission_textures")]
    pub thickness_texture: Option<Handle<Image>>,
    #[doc(alias = "index_of_refraction")]
    #[doc(alias = "refraction_index")]
    #[doc(alias = "refractive_index")]
    pub ior: f32,
    #[doc(alias = "absorption_distance")]
    #[doc(alias = "extinction_distance")]
    pub attenuation_distance: f32,
    #[doc(alias = "absorption_color")]
    #[doc(alias = "extinction_color")]
    pub attenuation_color: Color,
    #[texture(9)]
    #[sampler(10)]
    #[dependency]
    pub normal_map_texture: Option<Handle<Image>>,
    pub flip_normal_map_y: bool,
    #[texture(7)]
    #[sampler(8)]
    #[dependency]
    pub occlusion_texture: Option<Handle<Image>>,
    pub double_sided: bool,
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    #[texture(11)]
    #[sampler(12)]
    #[dependency]
    pub depth_map: Option<Handle<Image>>,
    pub parallax_depth_scale: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
    pub max_parallax_layer_count: f32,
    pub opaque_render_method: OpaqueRendererMethod,
    pub deferred_lighting_pass_id: u8,
}

impl Default for CustomMaterial {
    fn default() -> Self {
        CustomMaterial {
            // White because it gets multiplied with texture values if someone uses
            // a texture.
            base_color: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: None,
            emissive: Color::BLACK,
            emissive_texture: None,
            // Matches Blender's default roughness.
            perceptual_roughness: 0.5,
            // Metallic should generally be set to 0.0 or 1.0.
            metallic: 0.0,
            metallic_roughness_texture: None,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see
            // <https://google.github.io/filament/Material%20Properties.pdf>
            reflectance: 0.5,
            diffuse_transmission: 0.0,
            #[cfg(feature = "pbr_transmission_textures")]
            diffuse_transmission_texture: None,
            specular_transmission: 0.0,
            #[cfg(feature = "pbr_transmission_textures")]
            specular_transmission_texture: None,
            thickness: 0.0,
            #[cfg(feature = "pbr_transmission_textures")]
            thickness_texture: None,
            ior: 1.5,
            attenuation_color: Color::WHITE,
            attenuation_distance: f32::INFINITY,
            occlusion_texture: None,
            normal_map_texture: None,
            flip_normal_map_y: false,
            double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            fog_enabled: true,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            depth_map: None,
            parallax_depth_scale: 0.1,
            max_parallax_layer_count: 16.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
            opaque_render_method: OpaqueRendererMethod::Auto,
            deferred_lighting_pass_id: DEFAULT_PBR_DEFERRED_LIGHTING_PASS_ID,
        }
    }
}

impl From<Color> for CustomMaterial {
    fn from(color: Color) -> Self {
        CustomMaterial {
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

impl From<Handle<Image>> for CustomMaterial {
    fn from(texture: Handle<Image>) -> Self {
        CustomMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
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
            color: material.base_color != Color::WHITE,
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct CustomMaterialUniform {
    pub base_color: Vec4,
    pub emissive: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub diffuse_transmission: f32,
    pub specular_transmission: f32,
    pub thickness: f32,
    pub ior: f32,
    pub attenuation_distance: f32,
    pub attenuation_color: Vec4,
    pub flags: u32,
    pub alpha_cutoff: f32,
    pub parallax_depth_scale: f32,
    pub max_parallax_layer_count: f32,
    pub max_relief_mapping_search_steps: u32,
    pub deferred_lighting_pass_id: u32,
}

impl AsBindGroupShaderType<CustomMaterialUniform> for CustomMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> CustomMaterialUniform {
        let mut flags = NONE;
        if self.base_color_texture.is_some() {
            flags |= BASE_COLOR_TEXTURE;
        }
        if self.emissive_texture.is_some() {
            flags |= EMISSIVE_TEXTURE;
        }
        if self.metallic_roughness_texture.is_some() {
            flags |= METALLIC_ROUGHNESS_TEXTURE;
        }
        if self.occlusion_texture.is_some() {
            flags |= OCCLUSION_TEXTURE;
        }
        if self.double_sided {
            flags |= DOUBLE_SIDED;
        }
        if self.unlit {
            flags |= UNLIT;
        }
        if self.fog_enabled {
            flags |= FOG_ENABLED;
        }
        if self.depth_map.is_some() {
            flags |= DEPTH_MAP;
        }
        #[cfg(feature = "pbr_transmission_textures")]
        {
            if self.specular_transmission_texture.is_some() {
                flags |= SPECULAR_TRANSMISSION_TEXTURE;
            }
            if self.thickness_texture.is_some() {
                flags |= THICKNESS_TEXTURE;
            }
            if self.diffuse_transmission_texture.is_some() {
                flags |= DIFFUSE_TRANSMISSION_TEXTURE;
            }
        }
        let has_normal_map = self.normal_map_texture.is_some();
        if has_normal_map {
            let normal_map_id = self.normal_map_texture.as_ref().map(|h| h.id()).unwrap();
            if let Some(texture) = images.get(normal_map_id) {
                match texture.texture_format {
                    // All 2-component unorm formats
                    TextureFormat::Rg8Unorm
                    | TextureFormat::Rg16Unorm
                    | TextureFormat::Bc5RgUnorm
                    | TextureFormat::EacRg11Unorm => {
                        flags |= TWO_COMPONENT_NORMAL_MAP;
                    }
                    _ => {}
                }
            }
            if self.flip_normal_map_y {
                flags |= FLIP_NORMAL_MAP_Y;
            }
        }

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

        if self.attenuation_distance.is_finite() {
            flags |= ATTENUATION_ENABLED;
        }
        CustomMaterialUniform {
            base_color: self.base_color.as_linear_rgba_f32().into(),
            emissive: self.emissive.as_linear_rgba_f32().into(),
            roughness: self.perceptual_roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            diffuse_transmission: self.diffuse_transmission,
            specular_transmission: self.specular_transmission,
            thickness: self.thickness,
            ior: self.ior,
            attenuation_distance: self.attenuation_distance,
            attenuation_color: self.attenuation_color.as_linear_rgba_f32().into(),
            flags,
            alpha_cutoff,
            parallax_depth_scale: self.parallax_depth_scale,
            max_parallax_layer_count: self.max_parallax_layer_count,
            max_relief_mapping_search_steps: 0,
            deferred_lighting_pass_id: self.deferred_lighting_pass_id as u32,
        }
    }
}

pub type CustomBundle = MaterialMeshBundle<CustomMaterial>;