//! Loads and renders a glTF file as a scene.

use std::f32::consts::{FRAC_PI_4, PI};
use bevy::{
    gltf::Gltf, pbr::DirectionalLightShadowMap, prelude::*
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_npr::toon::{ToonBundle, ToonMaterial, ToonShaderPlugin};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((DefaultPlugins,ToonShaderPlugin))
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, init)
        .add_systems(Update, animate_light_direction)
        .run();
}

fn parse_scene(scene_path: String) -> (String, usize) {
    if scene_path.contains('#') {
        let gltf_and_scene = scene_path.split('#').collect::<Vec<_>>();
        if let Some((last, path)) = gltf_and_scene.split_last() {
            if let Some(index) = last
                .strip_prefix("Scene")
                .and_then(|index| index.parse::<usize>().ok())
            {
                return (path.join("#"), index);
            }
        }
    }
    (scene_path, 0)
}
#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (file_path, _scene_index) = parse_scene("models/tuzi.glb".to_string());
    commands.insert_resource(MyAssetPack(asset_server.load(file_path)));
}

fn init(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    
){
    info!("init");
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(3.0, 4.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    },));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    commands.spawn(ToonBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: toon_materials.add(Color::SILVER.into()),
        ..default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

