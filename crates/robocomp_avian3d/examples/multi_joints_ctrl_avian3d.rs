//! Example demonstrating a robot chain with interleaved revolute and prismatic joints:
//! revolute → prismatic → revolute → prismatic, connecting five small cubes.
//! Allows controlling each joint on demand via the W/S + ShiftLeft keys.
//! Use Arrow Up/Down to cycle the active joint.

use avian3d::prelude::*;
use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use robocomp_avian3d::{
    RobocompAvianControllerPlugin, RobocompAvianPlugin,
    rc::{
        RcCollider, RcDisableSleep, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2,
        RcJointMotorControlConfig, RcLink, RcLinkRoot, RcPrismaticJointLimits,
        RcPrismaticJointMotor, RcRevoluteJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdJointLimits, RdMotorModel, RdMotorPosition, RdMotorVelocity, RdName},
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

const CUBE_SIZE: f32 = 1.0;
const HALF: f32 = CUBE_SIZE / 2.0;
/// Extra gap along the revolute joint axis between parent and child link faces.
const REVOLUTE_LINK_OFFSET: f32 = 0.5;

/// `RcJointKind` parent/child must match `RcLink.name` exactly — use these for both.
const LINKS: [&str; 5] = ["Link 1", "Link 2", "Link 3", "Link 4", "Link 5"];
const JOINTS: [&str; 4] = [
    "Revolute Joint 1",
    "Prismatic Joint 1",
    "Revolute Joint 2",
    "Prismatic Joint 2",
];

/// Revolute: `AccelerationBased` velocity motor — Avian wraps position error to [-π, π],
/// so continuous spin needs velocity control, not `SpringDamper` position control.
const REVOLUTE_MOTOR_DAMPING: f32 = 3.5;
const REVOLUTE_MOTOR_NORMAL_VEL: f32 = 10.;
const REVOLUTE_MOTOR_BOOST_VEL: f32 = 16.;
/// Prismatic: `SpringDamper` position control (see the standalone prismatic example).
const PRISMATIC_MOTOR_FREQUENCY: f32 = 9.;
const PRISMATIC_MOTOR_DAMPING_RATIO: f32 = 3.5;
const PRISMATIC_MOTOR_NORM_ANG_VEL: f32 = 8.;
const PRISMATIC_MOTOR_BOOST_ANG_VEL: f32 = 16.;

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
        info!("Robocomp multi joints control example running!")
    });

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let robot = multi_joint_robot(&mut meshes, &mut materials);
    commands.spawn_scene(bsn! {
        Name("Multi Joints Control Example Scene")
        Transform
        Visibility
        RcSceneRoot
        Children [
            ({robot}),
        ]
    });
}

fn multi_joint_robot(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Scene {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(CUBE_SIZE))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
        ..Default::default()
    });

    let link_0 = link(
        LINKS[0],
        RcRigidBody::Fixed,
        cube_mesh_hdl.clone(),
        cube_mat_hdl.clone(),
    );
    let link_1 = link(
        LINKS[1],
        RcRigidBody::Dynamic,
        cube_mesh_hdl.clone(),
        cube_mat_hdl.clone(),
    );
    let link_2 = link(
        LINKS[2],
        RcRigidBody::Dynamic,
        cube_mesh_hdl.clone(),
        cube_mat_hdl.clone(),
    );
    let link_3 = link(
        LINKS[3],
        RcRigidBody::Dynamic,
        cube_mesh_hdl.clone(),
        cube_mat_hdl.clone(),
    );
    let link_4 = link(LINKS[4], RcRigidBody::Dynamic, cube_mesh_hdl, cube_mat_hdl);

    let joint_0 = joint(JOINTS[0], Vec3::ZERO, Vec3::ZERO);
    let joint_1 = joint(JOINTS[1], Vec3::ZERO, Vec3::ZERO);
    let joint_2 = joint(JOINTS[2], Vec3::ZERO, Vec3::ZERO);
    let joint_3 = joint(JOINTS[3], Vec3::ZERO, Vec3::ZERO);

    let revolute_kind_0 = scene_component(revolute_kind(LINKS[0], LINKS[1]));
    let revolute_kind_2 = scene_component(revolute_kind(LINKS[2], LINKS[3]));
    let prismatic_kind_1 = scene_component(prismatic_kind(LINKS[1], LINKS[2]));
    let prismatic_kind_3 = scene_component(prismatic_kind(LINKS[3], LINKS[4]));
    let revolute_control_0 = scene_component(revolute_control());
    let revolute_control_2 = scene_component(revolute_control());
    let prismatic_control_1 = scene_component(prismatic_control());
    let prismatic_control_3 = scene_component(prismatic_control());
    let prismatic_limits_1 = scene_component(RcPrismaticJointLimits(RdJointLimits {
        min: 0.,
        max: CUBE_SIZE,
    }));
    let prismatic_limits_3 = scene_component(RcPrismaticJointLimits(RdJointLimits {
        min: 0.,
        max: CUBE_SIZE * 2.,
    }));

    let link_0_translation = Vec3::new(0., HALF, 0.);
    let joint_0_translation = Vec3::new(0., CUBE_SIZE, 0.);
    let link_1_translation = Vec3::new(0., CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET, 0.);
    let joint_1_translation = Vec3::new(HALF, CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET, 0.);
    let link_2_translation = Vec3::new(
        CUBE_SIZE + HALF,
        CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET,
        0.,
    );
    let joint_2_translation = Vec3::new(
        CUBE_SIZE + HALF,
        CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET + HALF,
        0.,
    );
    let link_3_translation = Vec3::new(
        CUBE_SIZE + HALF,
        CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET + HALF + HALF + REVOLUTE_LINK_OFFSET,
        0.,
    );
    let joint_3_translation = Vec3::new(
        CUBE_SIZE * 2. + HALF,
        CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET + HALF + HALF + REVOLUTE_LINK_OFFSET,
        0.,
    );
    let link_4_translation = Vec3::new(
        CUBE_SIZE + HALF + CUBE_SIZE,
        CUBE_SIZE + HALF + REVOLUTE_LINK_OFFSET + HALF + HALF + REVOLUTE_LINK_OFFSET,
        0.,
    );

    bsn! {
        Name("Multi Joint Robot")
        RcRobotRoot
        Transform
        Visibility
        Children [
            (
                {link_0}
                RcLinkRoot
                Transform { translation: {link_0_translation} }
            ),
            (
                // Create a revolute joint between LINKS[0] and LINKS[1].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                {joint_0}
                {revolute_kind_0}
                RcRevoluteJointMotor {
                    model: {RdMotorModel::AccelerationBased {
                        stiffness: 0.,
                        damping: REVOLUTE_MOTOR_DAMPING,
                    }},
                    velocity: {Some(RdMotorVelocity { target: 0. })},
                    position: None,
                    max_force: None,
                }
                // Configuration for the motor control for the revolute joint...
                {revolute_control_0}
                Transform { translation: {joint_0_translation} }
            ),
            (
                {link_1}
                Transform { translation: {link_1_translation} }
                RcDisableSleep
            ),
            (
                // Create a prismatic joint between LINKS[1] and LINKS[2].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                {joint_1}
                {prismatic_kind_1}
                RcPrismaticJointMotor {
                    model: {RdMotorModel::SpringDamper {
                        frequency: PRISMATIC_MOTOR_FREQUENCY,
                        damping_ratio: PRISMATIC_MOTOR_DAMPING_RATIO,
                    }},
                    velocity: None,
                    position: {Some(RdMotorPosition { target: 0. })},
                    max_force: None,
                }
                {prismatic_limits_1}
                // Configuration for the motor control for the prismatic joint...
                {prismatic_control_1}
                Transform { translation: {joint_1_translation} }
            ),
            (
                {link_2}
                Transform { translation: {link_2_translation} }
                RcDisableSleep
            ),
            (
                // Create a revolute joint between LINKS[2] and LINKS[3].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                {joint_2}
                {revolute_kind_2}
                RcRevoluteJointMotor {
                    model: {RdMotorModel::AccelerationBased {
                        stiffness: 0.,
                        damping: REVOLUTE_MOTOR_DAMPING,
                    }},
                    velocity: {Some(RdMotorVelocity { target: 0. })},
                    position: None,
                    max_force: None,
                }
                // Configuration for the motor control for the revolute joint...
                {revolute_control_2}
                Transform { translation: {joint_2_translation} }
            ),
            (
                {link_3}
                Transform { translation: {link_3_translation} }
                RcDisableSleep
            ),
            (
                // Create a prismatic joint between LINKS[3] and LINKS[4].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                {joint_3}
                {prismatic_kind_3}
                RcPrismaticJointMotor {
                    model: {RdMotorModel::SpringDamper {
                        frequency: PRISMATIC_MOTOR_FREQUENCY,
                        damping_ratio: PRISMATIC_MOTOR_DAMPING_RATIO,
                    }},
                    velocity: None,
                    position: {Some(RdMotorPosition { target: 0. })},
                    max_force: None,
                }
                {prismatic_limits_3}
                // Configuration for the motor control for the prismatic joint...
                {prismatic_control_3}
                Transform { translation: {joint_3_translation} }
            ),
            (
                {link_4}
                Transform { translation: {link_4_translation} }
                RcDisableSleep
            ),
        ]
    }
}

fn revolute_kind(parent: &'static str, child: &'static str) -> RcJointKind {
    RcJointKind::Revolute {
        parent: RdName::new(parent),
        child: RdName::new(child),
        axis: Vec3::Y,
    }
}

fn prismatic_kind(parent: &'static str, child: &'static str) -> RcJointKind {
    RcJointKind::Prismatic {
        parent: RdName::new(parent),
        child: RdName::new(child),
        axis: Vec3::X,
    }
}

fn revolute_control() -> RcJointMotorControlConfig {
    RcJointMotorControlConfig::Velocity {
        normal_vel: REVOLUTE_MOTOR_NORMAL_VEL,
        boost_vel: REVOLUTE_MOTOR_BOOST_VEL,
    }
}

fn prismatic_control() -> RcJointMotorControlConfig {
    RcJointMotorControlConfig::Position {
        norm_ang_vel: PRISMATIC_MOTOR_NORM_ANG_VEL,
        boost_ang_vel: PRISMATIC_MOTOR_BOOST_ANG_VEL,
    }
}

/// Insert a component value that `bsn!` cannot construct directly.
fn scene_component<T>(value: T) -> impl Scene
where
    T: Component + Clone + Send + Sync + 'static,
{
    bsn! {
        template(move |_ctx| Ok(value.clone()))
    }
}

fn link(
    name: &'static str,
    rigid_body: RcRigidBody,
    mesh_hdl: Handle<Mesh>,
    material_hdl: Handle<StandardMaterial>,
) -> impl Scene {
    bsn! {
        Name({name})
        RcLink {
            name: {RdName::new(name)},
            rigid_body: {rigid_body},
        }
        Visibility
        RcCollider { keep_mesh: true }
        Children [
            (
                Name({format!("{} Mesh", name)})
                Transform
                Visibility
                Mesh3d({mesh_hdl})
                MeshMaterial3d<StandardMaterial>({material_hdl})
            ),
        ]
    }
}

fn joint(name: &'static str, local_anchor1: Vec3, local_anchor2: Vec3) -> impl Scene {
    bsn! {
        Name({format!("{} Joint", name)})
        Visibility
        RcJoint {
            name: {RdName::new(name)},
        }
        Children [
            (
                Name({format!("{} local anchor 1", name)})
                Transform { translation: {local_anchor1} }
                Visibility
                RcJointLocalAnchor1
            ),
            (
                Name({format!("{} local anchor 2", name)})
                Transform { translation: {local_anchor2} }
                Visibility
                RcJointLocalAnchor2
            ),
        ]
    }
}
