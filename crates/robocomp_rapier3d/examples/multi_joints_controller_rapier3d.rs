//! Example demonstrating a robot chain with interleaved revolute and prismatic joints:
//! revolute → prismatic → revolute → prismatic, connecting five small cubes.
//! Allows controlling each joint on demand via the W/S + ShiftLeft keys.
//! Use Arrow Up/Down to cycle the active joint.

use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp_rapier3d::{
    RobocompRapierControllerPlugin, RobocompRapierPlugin,
    rc::{
        RcCollider, RcDisableSleep, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2,
        RcJointMotorControlConfig, RcLink, RcLinkRoot, RcPrismaticJointLimits,
        RcPrismaticJointMotor, RcRevoluteJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdJointLimits, RdMotorModel, RdMotorVelocity, RdName},
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

const CUBE_SIZE: f32 = 1.0;
const HALF: f32 = CUBE_SIZE / 2.0;

/// `RcJointKind` parent/child must match `RcLink.name` exactly — use these for both.
const LINKS: [&str; 5] = ["Link 1", "Link 2", "Link 3", "Link 4", "Link 5"];
const JOINTS: [&str; 4] = [
    "Revolute Joint 1",
    "Prismatic Joint 1",
    "Revolute Joint 2",
    "Prismatic Joint 2",
];

/// Motor control tuned for unit-sized cubes and the multi-link chain's added inertia.
const MOTOR_CONTROL_NORMAL_VEL: f32 = 10.;
const MOTOR_CONTROL_NORMAL_FACTOR: f32 = 125.;
const MOTOR_CONTROL_BOOST_VEL: f32 = 20.;
const MOTOR_CONTROL_BOOST_FACTOR: f32 = 200.;

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompRapierPlugin,
        RobocompRapierControllerPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || {
        info!("Robocomp multi joints controller example running!")
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
            Name::new("Multi Joints Controller Example Scene"),
            Transform::default(),
            Visibility::default(),
            RcSceneRoot, // Scene Root for Robocomp
        ))
        .with_child(multi_joint_robot(&mut meshes, &mut materials));
}

fn multi_joint_robot(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Bundle {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(CUBE_SIZE))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
        ..Default::default()
    });

    (
        Name::new("Multi Joint Robot"),
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
                Transform::from_translation(Vec3::new(0., HALF, 0.)),
            ),
            (
                // Create a revolute joint between LINKS[0] and LINKS[1].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINTS[0], Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Revolute {
                    parent: RdName::new(LINKS[0]),
                    child: RdName::new(LINKS[1]),
                    axis: Vec3::Y,
                },
                RcRevoluteJointMotor {
                    model: RdMotorModel::default(),
                    velocity: Some(RdMotorVelocity {
                        target: 0.,
                        factor: 0.,
                    }),
                    position: None,
                    max_force: None,
                },
                // Configuration for the motor controller for the revolute joint...
                RcJointMotorControlConfig::Velocity {
                    normal_vel: MOTOR_CONTROL_NORMAL_VEL,
                    normal_factor: MOTOR_CONTROL_NORMAL_FACTOR,
                    boost_vel: MOTOR_CONTROL_BOOST_VEL,
                    boost_factor: MOTOR_CONTROL_BOOST_FACTOR,
                },
                Transform::from_translation(Vec3::new(0., CUBE_SIZE, 0.)),
            ),
            (
                link(
                    LINKS[1],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::new(0., CUBE_SIZE + HALF, 0.)),
                RcDisableSleep, // Keep the dynamic link awake for stable simulation.
            ),
            (
                // Create a prismatic joint between LINKS[1] and LINKS[2].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINTS[1], Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Prismatic {
                    parent: RdName::new(LINKS[1]),
                    child: RdName::new(LINKS[2]),
                    axis: Vec3::X,
                },
                RcPrismaticJointMotor {
                    model: RdMotorModel::AccelerationBased,
                    velocity: Some(RdMotorVelocity {
                        target: 0.,
                        factor: 0.,
                    }),
                    position: None,
                    max_force: None,
                },
                RcPrismaticJointLimits(RdJointLimits {
                    min: 0.,
                    max: CUBE_SIZE,
                }), // Limit the prismatic joint.
                // Configuration for the motor controller for the prismatic joint...
                RcJointMotorControlConfig::Velocity {
                    normal_vel: MOTOR_CONTROL_NORMAL_VEL,
                    normal_factor: MOTOR_CONTROL_NORMAL_FACTOR,
                    boost_vel: MOTOR_CONTROL_BOOST_VEL,
                    boost_factor: MOTOR_CONTROL_BOOST_FACTOR,
                },
                Transform::from_translation(Vec3::new(HALF, CUBE_SIZE + HALF, 0.)),
            ),
            (
                link(
                    LINKS[2],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::new(CUBE_SIZE + HALF, CUBE_SIZE + HALF, 0.)),
                RcDisableSleep, // Keep the dynamic link awake for stable simulation.
            ),
            (
                // Create a revolute joint between LINKS[2] and LINKS[3].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINTS[2], Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Revolute {
                    parent: RdName::new(LINKS[2]),
                    child: RdName::new(LINKS[3]),
                    axis: Vec3::Y,
                },
                RcRevoluteJointMotor {
                    model: RdMotorModel::default(),
                    velocity: Some(RdMotorVelocity {
                        target: 0.,
                        factor: 0.,
                    }),
                    position: None,
                    max_force: None,
                },
                // Configuration for the motor controller for the revolute joint...
                RcJointMotorControlConfig::Velocity {
                    normal_vel: MOTOR_CONTROL_NORMAL_VEL,
                    normal_factor: MOTOR_CONTROL_NORMAL_FACTOR,
                    boost_vel: MOTOR_CONTROL_BOOST_VEL,
                    boost_factor: MOTOR_CONTROL_BOOST_FACTOR,
                },
                Transform::from_translation(Vec3::new(
                    CUBE_SIZE + HALF,
                    CUBE_SIZE + HALF + HALF,
                    0.
                )),
            ),
            (
                link(
                    LINKS[3],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::new(
                    CUBE_SIZE + HALF,
                    CUBE_SIZE + HALF + CUBE_SIZE,
                    0.
                )),
                RcDisableSleep, // Keep the dynamic link awake for stable simulation.
            ),
            (
                // Create a prismatic joint between LINKS[3] and LINKS[4].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                joint(JOINTS[3], Vec3::ZERO, Vec3::ZERO),
                RcJointKind::Prismatic {
                    parent: RdName::new(LINKS[3]),
                    child: RdName::new(LINKS[4]),
                    axis: Vec3::X,
                },
                RcPrismaticJointMotor {
                    model: RdMotorModel::AccelerationBased,
                    velocity: Some(RdMotorVelocity {
                        target: 0.,
                        factor: 0.,
                    }),
                    position: None,
                    max_force: None,
                },
                RcPrismaticJointLimits(RdJointLimits {
                    min: 0.,
                    max: CUBE_SIZE,
                }), // Limit the prismatic joint.
                // Configuration for the motor controller for the prismatic joint...
                RcJointMotorControlConfig::Velocity {
                    normal_vel: MOTOR_CONTROL_NORMAL_VEL,
                    normal_factor: MOTOR_CONTROL_NORMAL_FACTOR,
                    boost_vel: MOTOR_CONTROL_BOOST_VEL,
                    boost_factor: MOTOR_CONTROL_BOOST_FACTOR,
                },
                Transform::from_translation(Vec3::new(
                    CUBE_SIZE + HALF + HALF,
                    CUBE_SIZE + HALF + CUBE_SIZE,
                    0.
                )),
            ),
            (
                link(
                    LINKS[4],
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone()
                ),
                Transform::from_translation(Vec3::new(
                    CUBE_SIZE + HALF + CUBE_SIZE,
                    CUBE_SIZE + HALF + CUBE_SIZE,
                    0.,
                )),
                RcDisableSleep, // Keep the dynamic link awake for stable simulation.
            ),
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
        Name::new(name),
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
