use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::prelude::*;

use robocomp::rc::{
    BelongsToRcSceneRoot, RcJointMotorControlConfig, RcLink, RcRobotGraph, RcRobotGraphJoint,
    RcRobotRoot, RcSceneRoot, RcSceneRootReady, RcSceneRootWRobotRootsDetached,
};

use crate::pre_processor_plugin::PreProcessorPluginSystemSet;

/// Plugin for controlling Robocomp robots with Avian.
pub struct RobocompAvianControllerPlugin;

impl Plugin for RobocompAvianControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActiveJointControlTracker>()
            .configure_sets(
                Update,
                ControllerPluginSystemSet.after(PreProcessorPluginSystemSet),
            )
            .configure_sets(
                FixedPostUpdate,
                ControllerMotorSystemSet
                    .after(PreProcessorPluginSystemSet)
                    .before(PhysicsSystems::Prepare),
            )
            .add_systems(
                Update,
                (setup_active_joint_tracker.before(active_joint_change_handler),)
                    .in_set(ControllerPluginSystemSet),
            )
            .add_systems(
                Update,
                active_joint_change_handler.in_set(ControllerPluginSystemSet),
            )
            .add_systems(
                FixedUpdate,
                joint_controller_system.in_set(ControllerMotorSystemSet),
            )
            .add_systems(
                Update,
                debug_render_active_joint
                    .in_set(ControllerPluginSystemSet)
                    .run_if(|| true),
            )
            .add_systems(Startup, || info!("ControllerPlugin started..."));
    }
}

/// Update-scheduled controller systems.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ControllerPluginSystemSet;

/// Fixed-update motor control, ordered before Avian physics prepare.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ControllerMotorSystemSet;

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

/// Tracks the currently controlled joint and other joints in the controller queue.
#[derive(Debug, Clone, Reflect, Component)]
pub struct ActiveJointControlTracker {
    active_joint_idx: usize,
    joints_list: Vec<Entity>,
}

impl ActiveJointControlTracker {
    pub fn active(&self) -> Option<&Entity> {
        self.joints_list.get(self.active_joint_idx)
    }

    pub fn add(&mut self, joint: Entity) -> Option<&Entity> {
        self.joints_list.push(joint);
        self.active_joint_idx = self.joints_list.len() - 1;
        self.joints_list.last()
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, joint: Entity) -> Option<usize> {
        if self.joints_list.is_empty() {
            return None;
        }
        self.joints_list.retain(|&j| j != joint);
        self.active_joint_idx %= self.joints_list.len();
        Some(self.active_joint_idx)
    }

    pub fn next(&mut self) -> Option<&Entity> {
        if self.joints_list.is_empty() {
            return None;
        }
        self.active_joint_idx = (self.active_joint_idx + 1) % self.joints_list.len();
        self.joints_list.get(self.active_joint_idx)
    }

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
    robot_roots: Query<&RcRobotGraph, With<RcRobotRoot>>,
    links: Query<&Transform, With<RcLink>>,
    mut gizmos: Gizmos,
) {
    for active_joint_tracker in active_joint_tracker.iter() {
        let Some(active_joint) = active_joint_tracker.active() else {
            continue;
        };
        let child_link = robot_roots
            .iter()
            .flat_map(|graph| graph.joints())
            .find(|joint| joint.joint_entity == *active_joint)
            .map(|joint| joint.to_link.clone());

        let Some(child_link_name) = child_link else {
            continue;
        };

        let Some(transform) = robot_roots.iter().find_map(|graph| {
            graph
                .links()
                .find(|link| link.link_name == child_link_name)
                .and_then(|link| links.get(link.link_entity).ok())
        }) else {
            continue;
        };

        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            1.,
            Color::from(Srgba::BLUE),
        );
    }
}

pub fn joint_controller_system(
    robot_roots: Query<(Entity, &ActiveJointControlTracker), With<RcRobotRoot>>,
    mut revolute_joints: Query<(&RcJointMotorControlConfig, &mut RevoluteJoint)>,
    mut prismatic_joints: Query<(&RcJointMotorControlConfig, &mut PrismaticJoint)>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
) {
    for (_robot_root_ent, active_joint_tracker) in robot_roots.iter() {
        let Some(joint_entity) = active_joint_tracker.active() else {
            continue;
        };

        if let Ok((config, mut joint)) = revolute_joints.get_mut(*joint_entity) {
            apply_revolute_motor_control(&key_input, config, &time, &mut joint.motor);
            continue;
        }

        if let Ok((config, mut joint)) = prismatic_joints.get_mut(*joint_entity) {
            apply_prismatic_motor_control(&key_input, config, &time, &mut joint.motor);
        }
    }
}

fn apply_revolute_motor_control(
    key_input: &ButtonInput<KeyCode>,
    config: &RcJointMotorControlConfig,
    time: &Time<Fixed>,
    motor: &mut AngularMotor,
) {
    match get_motor_control_args(key_input, config, time, motor.target_position) {
        MotorControlArgs::Position(target_pos, stiffness, damping) => {
            motor.target_position = target_pos % TAU;
            motor.motor_model = MotorModel::AccelerationBased { stiffness, damping };
        }
        MotorControlArgs::Velocity(target_vel, _factor) => {
            motor.target_velocity = target_vel;
        }
    }
}

fn apply_prismatic_motor_control(
    key_input: &ButtonInput<KeyCode>,
    config: &RcJointMotorControlConfig,
    time: &Time<Fixed>,
    motor: &mut LinearMotor,
) {
    match get_motor_control_args(key_input, config, time, motor.target_position) {
        MotorControlArgs::Position(target_pos, stiffness, damping) => {
            motor.target_position = target_pos;
            motor.motor_model = MotorModel::AccelerationBased { stiffness, damping };
        }
        MotorControlArgs::Velocity(target_vel, _factor) => {
            motor.target_velocity = target_vel;
        }
    }
}

fn get_motor_control_args(
    key_input: &ButtonInput<KeyCode>,
    joint_motor_control_config: &RcJointMotorControlConfig,
    time: &Time<Fixed>,
    current_target_pos: f32,
) -> MotorControlArgs {
    let shift_pressed = key_input.pressed(KeyCode::ShiftLeft);
    let w_pressed = key_input.pressed(KeyCode::KeyW);
    let s_pressed = key_input.pressed(KeyCode::KeyS);

    match joint_motor_control_config {
        RcJointMotorControlConfig::Position {
            boost_ang_vel,
            norm_ang_vel,
            stiffness,
            damping,
        } => {
            let mut new_target_pos = current_target_pos;
            let new_velocity = if shift_pressed && (w_pressed || s_pressed) {
                *boost_ang_vel
            } else if w_pressed || s_pressed {
                *norm_ang_vel
            } else {
                0.
            };
            if w_pressed {
                new_target_pos += new_velocity * time.delta_secs();
            } else if s_pressed {
                new_target_pos -= new_velocity * time.delta_secs();
            }
            MotorControlArgs::Position(new_target_pos, *stiffness, *damping)
        }
        RcJointMotorControlConfig::Velocity {
            normal_vel,
            normal_factor,
            boost_vel,
            boost_factor,
        } => {
            let (mut new_target_vel, new_factor) = if shift_pressed && (w_pressed || s_pressed) {
                (*boost_vel, *boost_factor)
            } else if w_pressed || s_pressed {
                (*normal_vel, *normal_factor)
            } else {
                (0., 0.)
            };
            if s_pressed {
                new_target_vel *= -1.;
            }
            MotorControlArgs::Velocity(new_target_vel, new_factor)
        }
    }
}

#[derive(Debug, Clone)]
enum MotorControlArgs {
    Position(f32, f32, f32),
    Velocity(f32, f32),
}
