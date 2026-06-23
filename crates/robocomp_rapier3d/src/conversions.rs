//! Conversions from physics-agnostic robocomp types to Rapier builders.

use bevy_rapier3d::prelude::{
    FixedJointBuilder, JointAxis, MotorModel, PrismaticJointBuilder, RevoluteJointBuilder,
    RigidBody, SphericalJointBuilder,
};
use robocomp::{
    rc::RcRigidBody,
    rd::{
        RdFixedJointData, RdMotor, RdMotorModel, RdPrismaticJointData, RdRevoluteJointData,
        RdSphericalJointData,
    },
};

pub fn rigid_body_from_rc(rigid_body: RcRigidBody) -> RigidBody {
    match rigid_body {
        RcRigidBody::Fixed => RigidBody::Fixed,
        RcRigidBody::Dynamic => RigidBody::Dynamic,
    }
}

pub fn fixed_joint_builder_from_rd(value: RdFixedJointData) -> FixedJointBuilder {
    let RdFixedJointData {
        local_anchor1,
        local_anchor2,
    } = value;

    FixedJointBuilder::new()
        .local_anchor1(local_anchor1)
        .local_anchor2(local_anchor2)
}

fn apply_motor_to_revolute_builder(
    mut builder: RevoluteJointBuilder,
    motor: RdMotor,
) -> RevoluteJointBuilder {
    builder = builder.motor_model(rapier_motor_model_from_rd(&motor.model));
    if let Some(motor_velocity) = motor.velocity {
        builder = builder.motor_velocity(motor_velocity.target, motor_velocity.factor);
    }
    if let Some(motor_position) = motor.position {
        let (stiffness, damping) = stiffness_damping_from_rd(&motor.model);
        builder = builder.motor_position(motor_position.target, stiffness, damping);
    }
    if let Some(motor_max_force) = motor.max_force {
        builder = builder.motor_max_force(motor_max_force);
    }
    builder
}

fn apply_motor_to_prismatic_builder(
    mut builder: PrismaticJointBuilder,
    motor: RdMotor,
) -> PrismaticJointBuilder {
    builder = builder.motor_model(rapier_motor_model_from_rd(&motor.model));
    if let Some(motor_velocity) = motor.velocity {
        builder = builder.motor_velocity(motor_velocity.target, motor_velocity.factor);
    }
    if let Some(motor_position) = motor.position {
        let (stiffness, damping) = stiffness_damping_from_rd(&motor.model);
        builder = builder.motor_position(motor_position.target, stiffness, damping);
    }
    if let Some(motor_max_force) = motor.max_force {
        builder = builder.motor_max_force(motor_max_force);
    }
    builder
}

pub fn revolute_joint_builder_from_rd(value: RdRevoluteJointData) -> RevoluteJointBuilder {
    let RdRevoluteJointData {
        axis,
        local_anchor1,
        local_anchor2,
        motor,
        limits,
    } = value;

    let mut builder = RevoluteJointBuilder::new(axis)
        .local_anchor1(local_anchor1)
        .local_anchor2(local_anchor2);
    builder = apply_motor_to_revolute_builder(builder, motor);
    if let Some(limits) = limits {
        builder = builder.limits([limits.min, limits.max]);
    }

    builder
}

pub fn prismatic_joint_builder_from_rd(value: RdPrismaticJointData) -> PrismaticJointBuilder {
    let RdPrismaticJointData {
        axis,
        local_anchor1,
        local_anchor2,
        motor,
        limits,
    } = value;

    let mut builder = PrismaticJointBuilder::new(axis)
        .local_anchor1(local_anchor1)
        .local_anchor2(local_anchor2);
    builder = apply_motor_to_prismatic_builder(builder, motor);
    if let Some(limits) = limits {
        builder = builder.limits([limits.min, limits.max]);
    }

    builder
}

pub fn spherical_joint_builder_from_rd(value: RdSphericalJointData) -> SphericalJointBuilder {
    let RdSphericalJointData {
        local_anchor1,
        local_anchor2,
        motor_x,
        motor_y,
        motor_z,
        limit_ang_x,
        limit_ang_y,
        limit_ang_z,
    } = value;

    let mut builder = SphericalJointBuilder::new()
        .local_anchor1(local_anchor1)
        .local_anchor2(local_anchor2);
    [motor_x, motor_y, motor_z]
        .iter()
        .enumerate()
        .for_each(|(i, motor)| {
            if let Some(motor) = motor {
                let axis = match i {
                    0 => JointAxis::AngX,
                    1 => JointAxis::AngY,
                    2 => JointAxis::AngZ,
                    _ => unreachable!(),
                };
                builder = builder.motor_model(axis, rapier_motor_model_from_rd(&motor.model));
                if let Some(motor_velocity) = &motor.velocity {
                    builder = builder.motor_velocity(
                        axis,
                        motor_velocity.target,
                        motor_velocity.factor,
                    );
                }
                if let Some(motor_position) = &motor.position {
                    let (stiffness, damping) = stiffness_damping_from_rd(&motor.model);
                    builder = builder.motor_position(
                        axis,
                        motor_position.target,
                        stiffness,
                        damping,
                    );
                }
                if let Some(motor_max_force) = &motor.max_force {
                    builder = builder.motor_max_force(axis, *motor_max_force);
                }
            }
        });
    [limit_ang_x, limit_ang_y, limit_ang_z]
        .iter()
        .enumerate()
        .for_each(|(i, limits)| {
            if let Some(limits) = limits {
                let axis = match i {
                    0 => JointAxis::AngX,
                    1 => JointAxis::AngY,
                    2 => JointAxis::AngZ,
                    _ => unreachable!(),
                };
                builder = builder.limits(axis, [limits.min, limits.max]);
            }
        });

    builder
}

/// Maps `RdMotorModel` to Rapier's unit-variant `MotorModel`.
pub fn rapier_motor_model_from_rd(value: &RdMotorModel) -> MotorModel {
    match value {
        RdMotorModel::SpringDamper { .. } | RdMotorModel::AccelerationBased { .. } => {
            MotorModel::AccelerationBased
        }
        RdMotorModel::ForceBased { .. } => MotorModel::ForceBased,
    }
}

/// Returns stiffness/damping coefficients for Rapier's `motor_position`.
///
/// `SpringDamper` is approximated as acceleration-based spring coefficients;
/// behavior will differ from Avian's implicit spring-damper integrator.
pub fn stiffness_damping_from_rd(value: &RdMotorModel) -> (f32, f32) {
    match value {
        RdMotorModel::SpringDamper {
            frequency,
            damping_ratio,
        } => {
            let omega = 2.0 * std::f32::consts::PI * frequency;
            (omega * omega, 2.0 * damping_ratio * omega)
        }
        RdMotorModel::ForceBased { stiffness, damping }
        | RdMotorModel::AccelerationBased { stiffness, damping } => (*stiffness, *damping),
    }
}

/// Backwards-compatible alias for [`rapier_motor_model_from_rd`].
pub fn motor_model_from_rd(value: &RdMotorModel) -> MotorModel {
    rapier_motor_model_from_rd(value)
}
