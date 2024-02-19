//! Loads and renders a glTF file as a scene.

use bevy::{gltf::{self, Gltf}, pbr::DirectionalLightShadowMap, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_npr::toon::{ToonBundle, ToonMaterial, ToonShaderPlugin};
use std::f32::consts::{FRAC_PI_4, PI};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((DefaultPlugins, ToonShaderPlugin))
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, init)
        .add_systems(Update, animate_light_direction)
        .run();
}

#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("setup");
    commands.insert_resource(MyAssetPack(
        asset_server.load("models/tuzi.glb".to_string()),
    ));
}

fn init(
    mut commands: Commands,
    mut _standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_pack: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
) {
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

    if let Some(gltf) = assets_gltf.get(&asset_pack.0) {
        commands.spawn(SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..Default::default()
        });
    }

    // commands.spawn(ToonBundle {
    //     mesh: meshes.add(Mesh::from(shape::Capsule::default())),
    //     transform: Transform::from_xyz(1.0, 1.0, 1.0),
    //     material: toon_materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
    //     ..default()
    // });

    commands.spawn(ToonBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: toon_materials.add(Color::SILVER.into()),
        ..default()
    }).with_children(|parent| {
        parent.spawn(ToonBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule::default())),
            material: toon_materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(1.0, 1.0, 1.0),
            ..Default::default()
        });
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
