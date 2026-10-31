//! Example demonstrating a robot with a prismatic joint (w/ limits) between two cubes.
//! Allows controlling the prismatic joint via the W/S + ShiftLeft Keys.

use avian3d::prelude::*;
use bevy::{color::palettes::tailwind::BLUE_400, prelude::*};
use robocomp_avian3d::{
    RobocompAvianControllerPlugin, RobocompAvianPlugin,
    rc::{
        RcCollider, RcDisableSleep, RcJoint, RcJointKind, RcJointLocalAnchor1, RcJointLocalAnchor2,
        RcJointMotorControlConfig, RcLink, RcLinkRoot, RcPrismaticJointLimits,
        RcPrismaticJointMotor, RcRigidBody, RcRobotRoot, RcSceneRoot,
    },
    rd::{RdJointLimits, RdMotorModel, RdMotorPosition, RdName},
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
        info!("Robocomp prismatic control example running!")
    });

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let robot = cube_revolute_cube_robot(&mut meshes, &mut materials);
    commands.spawn_scene(bsn! {
        Name("Prismatic Control Example Scene")
        Transform
        Visibility
        RcSceneRoot
        Children [
            ({robot}),
        ]
    });
}

fn cube_revolute_cube_robot(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Scene {
    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
        ..Default::default()
    });

    let base_link = link(
        LINKS[0],
        RcRigidBody::Fixed,
        cube_mesh_hdl.clone(),
        cube_mat_hdl.clone(),
    );
    let moving_link = link(LINKS[1], RcRigidBody::Dynamic, cube_mesh_hdl, cube_mat_hdl);
    let joint_scene = joint(JOINT, Vec3::ZERO, Vec3::ZERO);
    let joint_kind = scene_component(RcJointKind::Prismatic {
        parent: RdName::new(LINKS[0]),
        child: RdName::new(LINKS[1]),
        axis: Vec3::X,
    });
    let joint_limits = scene_component(RcPrismaticJointLimits(RdJointLimits { min: 0., max: 2.5 }));
    let motor_control = scene_component(RcJointMotorControlConfig::Position {
        norm_ang_vel: 4.,
        boost_ang_vel: 8.,
    });
    let base_translation = Vec3::new(0., 0.5, 0.);
    let joint_translation = Vec3::new(1., 0.5, 0.);
    let moving_translation = Vec3::new(2., 0.5, 0.);

    bsn! {
        Name("Cube Prismatic Cube Robot")
        RcRobotRoot
        Transform
        Visibility
        Children [
            (
                {base_link}
                RcLinkRoot
                Transform { translation: {base_translation} }
            ),
            (
                // Create a prismatic joint between LINKS[0] and LINKS[1].
                // Local anchors can usually be left to zero ie. on the same position as the joint entity,
                // but can be adjusted (in local space of the respective link) for different joint configurations.
                {joint_scene}
                {joint_kind}
                RcPrismaticJointMotor {
                    // Avian position motors are stable with SpringDamper (see Avian's joint_motors_3d
                    // example). Roughly equivalent to the Rapier example's AccelerationBased
                    // { stiffness: 5000., damping: 650. }.
                    model: {RdMotorModel::SpringDamper {
                        frequency: 11.25,
                        damping_ratio: 4.6,
                    }},
                    velocity: None,
                    position: {Some(RdMotorPosition { target: 0. })},
                    max_force: None,
                }
                {joint_limits}
                // Configuration for the motor control for the prismatic joint...
                {motor_control}
                Transform { translation: {joint_translation} }
            ),
            (
                {moving_link}
                Transform { translation: {moving_translation} }
                RcDisableSleep
            ),
        ]
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
        Name({format!("{} Link", name)})
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
