//! Example demonstrating a simple robot with a revolute joint between two cubes.

use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp::{
    RobocompPlugin,
    rc::{
        RcCollider, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2, RcLink,
        RcLinkRoot, RcRevoluteJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdMotorModel, RdMotorVelocity, RdName},
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
    app.add_systems(Startup, || info!("Robocomp revolute example running!"));

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
            Name::new("Revolute Example Scene"),
            Transform::default(),
            Visibility::default(),
            RcSceneRoot, // Scene Root for Robocomp
        ))
        .with_child(cube_revolute_cube_robot(&mut meshes, &mut materials));
}

fn cube_revolute_cube_robot(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Bundle {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
        ..Default::default()
    });
    let offset = 0.01;

    (
        Name::new("Cube Revolute Cube Robot"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![
            (
                link(
                    "Cube",
                    RcRigidBody::Fixed,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::Y * 0.5),
            ),
            (
                joint(
                    "Revolute Joint",
                    Vec3::Y * (1.0 + offset),
                    Vec3::Y * (1.0 + offset)
                ),
                RcJointKind::Revolute {
                    parent: RdName::new("Cube"),
                    child: RdName::new("Revolute Cube"),
                    axis: Vec3::Y,
                },
                RcRevoluteJointMotor {
                    model: RdMotorModel::default(),
                    velocity: Some(RdMotorVelocity {
                        target: 1.,
                        factor: 1.,
                    }),
                    position: None,
                    max_force: None,
                },
                Transform::from_translation(Vec3::Y * (1.0 + offset)),
            ),
            (
                link(
                    "Revolute Cube",
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::Y * (1.5 + offset)),
            )
        ],
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

fn joint(name: &'static str, local_anchor1: Vec3, local_anchor2: Vec3) -> impl Bundle {
    (
        Name::new(format!("{} Joint", name)),
        Visibility::default(),
        RcJoint {
            name: RdName::new(name),
            ..Default::default()
        },
        children![
            (
                Name::new(format!("{} local anchor 1", name)),
                Transform::from_translation(local_anchor1),
                Visibility::default(),
                RcJointLocalAnchor1,
            ),
            (
                Name::new(format!("{} local anchor 2", name)),
                Transform::from_translation(local_anchor2),
                Visibility::default(),
                RcJointLocalAnchor2,
            ),
        ],
    )
}
