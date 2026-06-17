//! A sample example demonstrating a single cube with a fixed rigid body and collider.

use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp::{
    RobocompPlugin,
    rc::{RcCollider, RcLink, RcLinkRoot, RcRigidBody, RcRobotRoot, RcSceneRoot},
    rd::RdName,
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || info!("Robocomp sample example running!"));

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scene...
    commands
        .spawn((
            Name::new("Sample Example Scene"),
            Transform::default(),
            Visibility::default(),
            RcSceneRoot, // Scene Root for Robocomp
        ))
        .with_child(cube_robot(&mut meshes, &mut materials));
}

fn cube_robot(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> impl Bundle {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
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
            RcLinkRoot, // Mark the first link as the root link of the robot
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
