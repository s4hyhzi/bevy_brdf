//! Loads and renders a glTF file as a scene.

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_npr::toon::{ToonBundle, ToonMaterial, ToonShaderPlugin};
use std::f32::consts::{FRAC_PI_4, PI};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((DefaultPlugins, ToonShaderPlugin))
        // .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PreStartup, setup)
        .add_systems(PreUpdate, update_materials)
        .add_systems(Update, animate_light_direction)
        .run();
}

fn setup(
    mut commands: Commands,
    mut _standard_materials: ResMut<Assets<StandardMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    info!("init");
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3.0, 4.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/tuzi.glb#Scene0".to_string()),
        ..Default::default()
    });

    commands.spawn(ToonBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: toon_materials.add(Color::SILVER),
        ..default()
    });
}

fn update_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ToonMaterial>>,
    spheres: Query<(Entity, &Handle<StandardMaterial>, &Name)>,
) {
    for sphere in spheres.iter() {
        let (_entity, material, _name) = sphere;
        // info!(
        //     "update_materials, entity: {:?}, material: {:?}, name: {:?}",
        //     entity, material, name
        // );
        let material_path = material.path();
        match material_path {
            Some(path) => {
                if path.to_string().contains("models/tuzi.glb#Material0") {
                    info!("update_materials, material: {:?}", material);

                    commands
                        .entity(sphere.0)
                        .remove::<Handle<StandardMaterial>>();
                    commands
                        .entity(sphere.0)
                        .insert(materials.add(Color::SILVER));
                }
            }
            None => {}
        }
    }
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
