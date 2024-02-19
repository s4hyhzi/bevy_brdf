use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_wgsl::toon::{ToonBundle, ToonMaterial, ToonShaderPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ToonShaderPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>, 
    _asset_server: Res<AssetServer>,
) {
    // cube
    commands.spawn(ToonBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
        transform: Transform::from_xyz(1.0, 1.0, 1.0),
        material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: standard_materials.add(Color::SILVER.into()),
        ..default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Capsule::default())),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     material: standard_materials.add(Color::rgb(0.5, 0.5, 0.3).into()),
    //     ..default()
    // });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(2.0, 2.0, 2.0),
            rotation: Quat::from_euler(EulerRot::XYZ, -PI / 4., PI / 6., 0.),
            ..default()
        },
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}