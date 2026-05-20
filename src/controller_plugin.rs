use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::{
    na::DVector,
    parry::math::Pose3,
    prelude::*,
    rapier::prelude::{InverseKinematicsOption, JointLimits, JointMotor, MultibodyLink},
};
// use common_utils::inspector::toggle_inspector_is_active;

use crate::rc::{
    BelongsToRcSceneRoot, RcSceneRoot, RcSceneRootReady, RcSceneRootWRobotRootsDetached,
};

use super::{
    pre_processor_plugin::PreProcessorPluginSystemSet,
    rc::{
        RcIkAffectedLinks, RcIkTarget, RcIkTargetSource, RcJointMotorControlConfig, RcLink,
        RcRobotGraph, RcRobotGraphJoint, RcRobotRoot,
    },
};

/// Plugin for Robot Controlling.
pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app // App
            // System Set...
            .configure_sets(
                Update,
                ControllerPluginSystemSet
                    .after(PreProcessorPluginSystemSet)
                    .before(PhysicsSet::SyncBackend),
            )
            // Systems...
            .add_systems(
                Update,
                (setup_active_joint_tracker.before(active_joint_change_handler),)
                    .in_set(ControllerPluginSystemSet),
            )
            .add_systems(
                Update,
                (apply_ik_target_tracking, active_joint_change_handler)
                    .in_set(ControllerPluginSystemSet),
            )
            .add_systems(
                FixedUpdate,
                impulse_joint_controller_system.in_set(ControllerPluginSystemSet),
            )
            // Debug Systems...
            // .add_systems(
            //     Update,
            //     debug_render_active_joint
            //         .in_set(ControllerPluginSystemSet)
            //         .run_if(toggle_inspector_is_active),
            // )
            .add_systems(Startup, || info!("ControllerPlugin started..."));
    }
}

/// The Plugin's own system set.
///
/// Used to apply common run conditions based on state etc to
/// a common set of systems.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ControllerPluginSystemSet;

/// Should run only when all pre-processing is complete. ie. after [`RcSceneRootWRobotRootsDetached`]
/// is added on [`RcSceneRoot`].
pub fn setup_active_joint_tracker(
    mut commands: Commands,
    scene_roots: Query<
        Entity,
        (
            With<RcSceneRoot>,
            With<RcSceneRootReady>,
            Added<RcSceneRootWRobotRootsDetached>,
        ),
    >,
    robot_roots: Query<
        (Entity, &BelongsToRcSceneRoot, &RcRobotGraph),
        (With<RcRobotRoot>, Without<ActiveJointControlTracker>),
    >,
) {
    for scene_root in scene_roots.iter() {
        for (robot_root_ent, &BelongsToRcSceneRoot(cur_scene_root), robot_graph) in
            robot_roots.iter()
        {
            if cur_scene_root != scene_root {
                continue;
            }
            let mut active_joint_control_tracker = ActiveJointControlTracker {
                active_joint_idx: 0,
                joints_list: Vec::new(),
            };
            for RcRobotGraphJoint { joint_entity, .. } in robot_graph.joints() {
                active_joint_control_tracker.add(*joint_entity);
            }
            commands
                .entity(robot_root_ent)
                .insert(active_joint_control_tracker);
        }
    }
}

/// Tracks the currently controlled joint and other joints in controller queue.
#[derive(Debug, Clone, Reflect, Component)]
pub struct ActiveJointControlTracker {
    /// The index of the currently active joint.
    active_joint_idx: usize,
    /// List of joints that are in the controller queue.
    joints_list: Vec<Entity>,
}

impl ActiveJointControlTracker {
    /// Get the active joint.
    pub fn active(&self) -> Option<&Entity> {
        self.joints_list.get(self.active_joint_idx)
    }

    /// Add a joint to the controller queue. Sets it as the active joint.
    ///
    /// Returns the last joint (usually the added) in the queue.
    pub fn add(&mut self, joint: Entity) -> Option<&Entity> {
        self.joints_list.push(joint);
        self.active_joint_idx = self.joints_list.len() - 1;

        self.joints_list.last()
    }

    /// Remove a joint from the controller queue. Sets the next joint as active.
    ///
    /// Returns the index of the active joint.
    #[allow(dead_code)]
    pub fn remove(&mut self, joint: Entity) -> Option<usize> {
        if self.joints_list.is_empty() {
            return None;
        }
        self.joints_list.retain(|&j| j != joint);
        self.active_joint_idx %= self.joints_list.len();

        Some(self.active_joint_idx)
    }

    /// Activate the next joint in the controller queue.
    ///
    /// Returns the active joint.
    pub fn next(&mut self) -> Option<&Entity> {
        if self.joints_list.is_empty() {
            return None;
        }
        self.active_joint_idx = (self.active_joint_idx + 1) % self.joints_list.len();

        self.joints_list.get(self.active_joint_idx)
    }

    /// Activate the previous joint in the controller queue.
    ///
    /// Returns the active joint.
    pub fn prev(&mut self) -> Option<&Entity> {
        if self.joints_list.is_empty() {
            return None;
        }
        self.active_joint_idx =
            (self.active_joint_idx + self.joints_list.len() - 1) % self.joints_list.len();

        self.joints_list.get(self.active_joint_idx)
    }
}

pub fn active_joint_change_handler(
    mut active_joint_tracker: Query<&mut ActiveJointControlTracker>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if key_input.just_pressed(KeyCode::ArrowDown) {
        for mut active_joint_tracker in active_joint_tracker.iter_mut() {
            active_joint_tracker.prev();
        }
    } else if key_input.just_pressed(KeyCode::ArrowUp) {
        for mut active_joint_tracker in active_joint_tracker.iter_mut() {
            active_joint_tracker.next();
        }
    }
}

pub fn debug_render_active_joint(
    active_joint_tracker: Query<&ActiveJointControlTracker>,
    joints: Query<&Transform, With<RcLink>>,
    mut gizmos: Gizmos,
) {
    for active_joint_tracker in active_joint_tracker.iter() {
        if let Some(active_joint) = active_joint_tracker.active() {
            let Ok(transform) = joints.get(*active_joint) else {
                continue;
            };
            gizmos.sphere(
                Isometry3d::from_translation(transform.translation),
                1.,
                Color::from(Srgba::BLUE),
            );
        }
    }
}

pub fn apply_ik_target_tracking(
    targets: Query<(Entity, &RcIkTarget, &GlobalTransform)>,
    source_links: Query<
        (
            &RcIkTargetSource,
            &RapierMultibodyJointHandle,
            Option<&RcIkAffectedLinks>,
        ),
        With<RcLink>,
    >,
    other_links: Query<(&RcLink, &RapierRigidBodyHandle), Without<RcIkTargetSource>>,
    mut write_rapier_context: WriteRapierContext,
) {
    let Ok(mut rapier_context) = write_rapier_context.single_mut() else {
        return;
    };
    for (_target_ent, ik_target, target_glob_transform) in targets.iter() {
        for (target_source, multibody_joint_hdl, affected_links) in source_links.iter() {
            // Proceed with only sources that are tracking this current target...
            if target_source.target != ik_target.name {
                continue;
            }
            let target_glob_translation = target_glob_transform.translation();
            // Get RB set for future use...
            let rigid_body_set = rapier_context.rigidbody_set.clone();
            // Get multibody and link id...
            let Some((multibody, link_id)) = rapier_context
                .joints
                .multibody_joints
                .get_mut(multibody_joint_hdl.0)
            else {
                continue;
            };

            let mut displacements = DVector::default();
            // Ensure our displacement vector has the right number of elements.
            if displacements.nrows() < multibody.ndofs() {
                displacements = DVector::zeros(multibody.ndofs());
            } else {
                displacements.fill(0.0);
            }
            let options = InverseKinematicsOption {
                damping: 20.,
                epsilon_angular: 1e-6,
                max_iters: 20,
                constrained_axes: JointAxesMask::LIN_AXES,
                ..Default::default()
            };
            let links_to_affect = other_links
                .iter()
                .filter_map(|(link, rigid_body_hdl)| {
                    if let Some(affected_links) = affected_links {
                        if affected_links.contains(&link.name) {
                            Some(*rigid_body_hdl)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let joint_can_move = |link: &MultibodyLink| {
                if links_to_affect.is_empty() {
                    true
                } else {
                    links_to_affect
                        .iter()
                        .any(|link_hdl| link_hdl.0 == link.rigid_body_handle())
                }
            };
            multibody.inverse_kinematics(
                &rigid_body_set.bodies,
                link_id,
                &options,
                &Pose3::from_translation(target_glob_translation),
                joint_can_move,
                &mut displacements,
            );
            multibody.apply_displacements(displacements.as_slice());
        }
    }
}

pub fn impulse_joint_controller_system(
    robot_roots: Query<(Entity, &ActiveJointControlTracker), With<RcRobotRoot>>,
    mut joints: Query<(&RcJointMotorControlConfig, &mut ImpulseJoint), With<RcLink>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (_robot_root_ent, active_joint_tracker) in robot_roots.iter() {
        let Some(generic_joint_entity) = active_joint_tracker.active() else {
            continue;
        };
        let Ok((joint_motor_control_config, mut joint)) = joints.get_mut(*generic_joint_entity)
        else {
            continue;
        };
        match joint.data {
            TypedJoint::FixedJoint(..) => {
                // No motor control for fixed joints...
            }
            TypedJoint::PrismaticJoint(..) => {
                let Some(prismatic) = joint.data.as_mut().as_prismatic_mut() else {
                    continue;
                };
                let Some(motor) = prismatic.motor() else {
                    continue;
                };
                match get_motor_control_args(&key_input, joint_motor_control_config, &time, motor) {
                    MotorControlArgs::Position(target_pos, stiffness, damping) => {
                        if let Some(&JointLimits { min, max, .. }) = prismatic.limits() {
                            prismatic.set_motor_position(
                                target_pos.clamp(min, max),
                                stiffness,
                                damping,
                            );
                        } else {
                            prismatic.set_motor_position(target_pos, stiffness, damping);
                        }
                    }
                    MotorControlArgs::Velocity(target_vel, factor) => {
                        prismatic.set_motor_velocity(target_vel, factor);
                    }
                }
            }
            TypedJoint::RevoluteJoint(..) => {
                let Some(revolute) = joint.data.as_mut().as_revolute_mut() else {
                    continue;
                };
                let Some(motor) = revolute.motor() else {
                    continue;
                };
                match get_motor_control_args(&key_input, joint_motor_control_config, &time, motor) {
                    MotorControlArgs::Position(target_pos, stiffness, damping) => {
                        if let Some(&JointLimits { min, max, .. }) = revolute.limits() {
                            revolute.set_motor_position(
                                target_pos.clamp(min, max),
                                stiffness,
                                damping,
                            );
                        } else {
                            // We % by TAU to keep the value within the range of 0-TAU...
                            revolute.set_motor_position(target_pos % TAU, stiffness, damping);
                        }
                    }
                    MotorControlArgs::Velocity(target_vel, factor) => {
                        revolute.set_motor_velocity(target_vel, factor);
                    }
                }
            }
            TypedJoint::SphericalJoint(..) => {
                let Some(_spherical) = joint.data.as_mut().as_spherical_mut() else {
                    continue;
                };
                // Todo: Implement spherical joint motor control...
            }
            _ => {}
        }
    }
}

/// Get the motor control arguments based on the key input, configuration, time and current motor state.
fn get_motor_control_args(
    key_input: &Res<'_, ButtonInput<KeyCode>>,
    joint_motor_control_config: &RcJointMotorControlConfig,
    time: &Res<'_, Time>,
    current_motor_state: &JointMotor,
) -> MotorControlArgs {
    // Capture key input...
    let shift_pressed = key_input.pressed(KeyCode::ShiftLeft);
    let w_pressed = key_input.pressed(KeyCode::KeyW);
    let s_pressed = key_input.pressed(KeyCode::KeyS);
    // Apply motor control based on the configuration...
    match joint_motor_control_config {
        RcJointMotorControlConfig::Position {
            boost_ang_vel,
            norm_ang_vel,
            stiffness,
            damping,
        } => {
            let mut new_target_pos = current_motor_state.target_pos;
            // Apply boost or normal velocity based on key input...
            let new_velocity = if shift_pressed {
                debug!("Key Shift pressed...");
                *boost_ang_vel
            } else if w_pressed || s_pressed {
                *norm_ang_vel
            } else {
                0.
            };
            // Apply direction based on key input...
            if w_pressed {
                debug!("Key W pressed...");
                new_target_pos += new_velocity * time.delta_secs();
            } else if s_pressed {
                debug!("Key S pressed...");
                new_target_pos -= new_velocity * time.delta_secs();
            } else {
                new_target_pos = current_motor_state.target_pos;
            }
            MotorControlArgs::Position(new_target_pos, *stiffness, *damping)
        }
        RcJointMotorControlConfig::Velocity {
            normal_vel,
            normal_factor,
            boost_vel,
            boost_factor,
        } => {
            let (mut new_target_vel, new_factor) = if shift_pressed {
                debug!("Key Shift pressed...");
                (*boost_vel, *boost_factor)
            } else if w_pressed || s_pressed {
                (*normal_vel, *normal_factor)
            } else {
                (0., 0.)
            };
            if w_pressed {
                debug!("Key W pressed...");
                new_target_vel *= 1.;
            } else if s_pressed {
                debug!("Key S pressed...");
                new_target_vel *= -1.;
            }
            MotorControlArgs::Velocity(new_target_vel, new_factor)
        }
    }
}

/// Arguments for controlling the motor.
#[derive(Debug, Clone)]
enum MotorControlArgs {
    Position(f32, f32, f32),
    Velocity(f32, f32),
}
