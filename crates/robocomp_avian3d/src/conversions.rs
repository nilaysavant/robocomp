//! Conversions from physics-agnostic robocomp types to Avian components.

use avian3d::{math::Vector, prelude::*};
use bevy::prelude::*;
use robocomp::{
    rc::RcRigidBody,
    rd::{
        RdFixedJointData, RdJointLimits, RdMotor, RdMotorModel, RdPrismaticJointData,
        RdRevoluteJointData, RdSphericalJointData,
    },
};

pub fn rigid_body_from_rc(rigid_body: RcRigidBody) -> RigidBody {
    match rigid_body {
        RcRigidBody::Fixed => RigidBody::Static,
        RcRigidBody::Dynamic => RigidBody::Dynamic,
    }
}

pub fn motor_model_from_rd(value: RdMotorModel) -> MotorModel {
    match value {
        RdMotorModel::SpringDamper {
            frequency,
            damping_ratio,
        } => MotorModel::SpringDamper {
            frequency,
            damping_ratio,
        },
        RdMotorModel::ForceBased { stiffness, damping } => {
            MotorModel::ForceBased { stiffness, damping }
        }
        RdMotorModel::AccelerationBased { stiffness, damping } => {
            MotorModel::AccelerationBased { stiffness, damping }
        }
    }
}

pub fn angular_motor_from_rd(motor: &RdMotor) -> AngularMotor {
    let mut angular_motor = AngularMotor::new(motor_model_from_rd(motor.model.clone()));

    if let Some(velocity) = &motor.velocity {
        angular_motor = angular_motor.with_target_velocity(velocity.target * velocity.factor);
    }
    if let Some(position) = &motor.position {
        angular_motor = angular_motor.with_target_position(position.target);
    }
    if let Some(max_force) = motor.max_force {
        angular_motor = angular_motor.with_max_torque(max_force);
    }

    angular_motor
}

pub fn linear_motor_from_rd(motor: &RdMotor) -> LinearMotor {
    let mut linear_motor = LinearMotor::new(motor_model_from_rd(motor.model.clone()));

    if let Some(velocity) = &motor.velocity {
        linear_motor = linear_motor.with_target_velocity(velocity.target * velocity.factor);
    }
    if let Some(position) = &motor.position {
        linear_motor = linear_motor.with_target_position(position.target);
    }
    if let Some(max_force) = motor.max_force {
        linear_motor = linear_motor.with_max_force(max_force);
    }

    linear_motor
}

pub fn fixed_joint_from_rd(
    parent: Entity,
    child: Entity,
    RdFixedJointData {
        local_anchor1,
        local_anchor2,
    }: RdFixedJointData,
) -> FixedJoint {
    FixedJoint::new(parent, child)
        .with_local_anchor1(vec3_to_vector(local_anchor1))
        .with_local_anchor2(vec3_to_vector(local_anchor2))
}

pub fn revolute_joint_from_rd(
    parent: Entity,
    child: Entity,
    RdRevoluteJointData {
        axis,
        local_anchor1,
        local_anchor2,
        motor,
        limits,
    }: RdRevoluteJointData,
) -> RevoluteJoint {
    let mut joint = RevoluteJoint::new(parent, child)
        .with_hinge_axis(vec3_to_vector(axis))
        .with_local_anchor1(vec3_to_vector(local_anchor1))
        .with_local_anchor2(vec3_to_vector(local_anchor2))
        .with_motor(angular_motor_from_rd(&motor));

    if let Some(RdJointLimits { min, max }) = limits {
        joint = joint.with_angle_limits(min, max);
    }

    joint
}

pub fn prismatic_joint_from_rd(
    parent: Entity,
    child: Entity,
    RdPrismaticJointData {
        axis,
        local_anchor1,
        local_anchor2,
        motor,
        limits,
    }: RdPrismaticJointData,
) -> PrismaticJoint {
    let mut joint = PrismaticJoint::new(parent, child)
        .with_slider_axis(vec3_to_vector(axis))
        .with_local_anchor1(vec3_to_vector(local_anchor1))
        .with_local_anchor2(vec3_to_vector(local_anchor2))
        .with_motor(linear_motor_from_rd(&motor));

    if let Some(RdJointLimits { min, max }) = limits {
        joint = joint.with_limits(min, max);
    }

    joint
}

pub fn spherical_joint_from_rd(
    parent: Entity,
    child: Entity,
    RdSphericalJointData {
        local_anchor1,
        local_anchor2,
        limit_ang_x,
        limit_ang_y,
        limit_ang_z,
        ..
    }: RdSphericalJointData,
) -> SphericalJoint {
    let mut joint = SphericalJoint::new(parent, child)
        .with_local_anchor1(vec3_to_vector(local_anchor1))
        .with_local_anchor2(vec3_to_vector(local_anchor2));

    // Best-effort mapping: Rapier per-axis limits → Avian swing/twist limits.
    if let Some(RdJointLimits { min, max }) = limit_ang_x {
        joint = joint.with_twist_limits(min, max);
    }
    if let Some(RdJointLimits { min, max }) = limit_ang_y {
        joint = joint.with_swing_limits(min, max);
    }
    if let Some(RdJointLimits { min, max }) = limit_ang_z {
        joint = joint.with_swing_limits(min, max);
    }

    joint
}

fn vec3_to_vector(value: Vec3) -> Vector {
    Vector::from(value)
}
