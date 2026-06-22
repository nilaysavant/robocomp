//! Example demonstrating a robot with a prismatic joint (w/ limits) between two cubes.
//! Allows controlling the prismatic joint via the W/S + ShiftLeft Keys.

use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use avian3d::prelude::*;
use robocomp_avian3d::{
    RobocompAvianControllerPlugin, RobocompAvianPlugin,
    rc::{
        RcCollider, RcDisableSleep, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2,
        RcJointMotorControlConfig, RcLink, RcLinkRoot, RcPrismaticJointLimits,
        RcPrismaticJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdJointLimits, RdMotorModel, RdMotorVelocity, RdName},
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

/// `RcJointKind` parent/child must match `RcLink.name` exactly — use these for both.
const LINKS: [&str; 2] = ["Cube", "Prismatic Cube"];
const JOINT: &str = "Prismatic Joint";

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add avian physics and robocomp plugins...
    app.add_plugins((
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
        RobocompAvianPlugin,
        RobocompAvianControllerPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || {
        info!("Robocomp prismatic controller example running!")
    });

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
            Name::new("Prismatic Controller Example Scene"),
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
        Name::new("Cube Prismatic Cube Robot"),
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
                Transform::from_translation(Vec3::new(0., 0.5, 0.)),
            ),
            (
                // Create a prismatic joint between LINKS[0] and LINKS[1].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINT, Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Prismatic {
                    parent: RdName::new(LINKS[0]),
                    child: RdName::new(LINKS[1]),
                    axis: Vec3::X,
                },
                RcPrismaticJointMotor {
                    model: RdMotorModel::AccelerationBased,
                    velocity: Some(RdMotorVelocity {
                        target: 0.,
                        factor: 0.
                    }),
                    position: None,
                    max_force: None,
                },
                RcPrismaticJointLimits(RdJointLimits { min: 0., max: 2.5 }), // Limit the prismatic joint.
                // Configuration for the motor controller for the prismatic joint...
                RcJointMotorControlConfig::Velocity {
                    normal_vel: 4.,
                    normal_factor: 10.,
                    boost_vel: 8.,
                    boost_factor: 10.0
                },
                Transform::from_translation(Vec3::new(1., 0.5, 0.)),
            ),
            (
                link(
                    LINKS[1],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::new(2., 0.5, 0.)),
                RcDisableSleep,
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
