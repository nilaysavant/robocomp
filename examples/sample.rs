//! A sample example demonstrating a single cube with a fixed rigid body and collider.

use bevy::{
    color::palettes::css::{GREEN, ORANGE, WHITE},
    light::light_consts::lux::CLEAR_SUNRISE,
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp::{
    RobocompPlugin,
    rc::{RcCollider, RcLink, RcLinkRoot, RcRigidBody, RcRobotRoot, RcSceneRoot},
    rd::RdName,
};

fn main() {
    let mut app = App::new();
    // Add Bevy, debug, camera plugins...
    app.add_plugins((
        DefaultPlugins,
        EguiPlugin::default(),
        WorldInspectorPlugin::default(),
    ));
    // Setup environment...
    app.insert_resource(GlobalAmbientLight {
        color: Color::from(WHITE),
        brightness: 500.,
        ..Default::default()
    });
    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || info!("Robocomp utils example running!"));

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera...
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::ONE * 5.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light...
    commands.spawn((
        Name::new("Sun Light"),
        DirectionalLight {
            color: Color::WHITE,
            illuminance: CLEAR_SUNRISE,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane...
    commands.spawn((
        Name::new("Ground Plane"),
        Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(5.0))))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::from(GREEN),
            perceptual_roughness: 0.9,
            ..Default::default()
        })),
    ));

    // Scene...
    commands
        .spawn((
            Name::new("Sample Scene"),
            Transform::default(),
            Visibility::default(),
            RcSceneRoot, // Scene Root for Robocomp
        ))
        .with_child(cube_robot(&mut meshes, &mut materials));
}

fn cube_robot(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> impl Bundle {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(ORANGE),
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    (
        Name::new("Cube Robot"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![(
            link(
                "Cube",
                RcRigidBody::default(),
                cube_mesh_hdl.clone(),
                cube_mat_hdl.clone()
            ),
            Transform::from_translation(Vec3::Y * 0.5),
        )],
    )
}

fn link(
    name: &'static str,
    rigid_body: RcRigidBody,
    mesh_hdl: Handle<Mesh>,
    material_hdl: Handle<StandardMaterial>,
) -> impl Bundle {
    (
        Name::new(format!("{} Link", name)),
        RcLink {
            name: RdName::new(name),
            rigid_body,
        },
        Visibility::default(),
        RcLinkRoot,
        RcCollider {
            keep_mesh: true,
            ..Default::default()
        },
        children![(
            Name::new(format!("{} Mesh", name)),
            Transform::default(),
            Visibility::default(),
            Mesh3d(mesh_hdl),
            MeshMaterial3d(material_hdl),
        )],
    )
}
