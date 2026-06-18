//! Example demonstrating a simple robot with a revolute joint between two cubes.

use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp_rapier::{
    RobocompRapierPlugin,
    rc::{
        RcCollider, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2, RcLink,
        RcLinkRoot, RcRevoluteJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdMotorModel, RdMotorVelocity, RdName},
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

/// `RcJointKind` parent/child must match `RcLink.name` exactly — use these for both.
const LINKS: [&str; 2] = ["Cube", "Revolute Cube"];
const JOINT: &str = "Revolute Joint";

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompRapierPlugin,
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

    (
        Name::new("Cube Revolute Cube Robot"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![
            (
                link(
                    LINKS[0],
                    RcRigidBody::Fixed,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                RcLinkRoot, // Mark the first link as the root link for the robot.
                Transform::from_translation(Vec3::Y * 0.5),
            ),
            (
                // Create a revolute joint between LINKS[0] and LINKS[1].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINT, Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Revolute {
                    parent: RdName::new(LINKS[0]),
                    child: RdName::new(LINKS[1]),
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
                Transform::from_translation(Vec3::Y * 1.5),
            ),
            (
                link(
                    LINKS[1],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::Y * 2.5),
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
