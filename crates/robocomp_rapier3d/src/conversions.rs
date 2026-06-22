//! Conversions from physics-agnostic robocomp types to Rapier builders.

use bevy_rapier3d::prelude::{
    FixedJointBuilder, JointAxis, MotorModel, PrismaticJointBuilder, RevoluteJointBuilder,
    RigidBody, SphericalJointBuilder,
};
use robocomp::{
    rc::RcRigidBody,
    rd::{
        RdFixedJointData, RdMotorModel, RdPrismaticJointData, RdRevoluteJointData,
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
        .local_anchor2(local_anchor2)
        .motor_model(motor_model_from_rd(motor.model));
    if let Some(motor_velocity) = motor.velocity {
        builder = builder.motor_velocity(motor_velocity.target, motor_velocity.factor);
    }
    if let Some(motor_position) = motor.position {
        builder = builder.motor_position(
            motor_position.target,
            motor_position.stiffness,
            motor_position.damping,
        );
    }
    if let Some(motor_max_force) = motor.max_force {
        builder = builder.motor_max_force(motor_max_force);
    }
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
        .local_anchor2(local_anchor2)
        .motor_model(motor_model_from_rd(motor.model));
    if let Some(motor_velocity) = motor.velocity {
        builder = builder.motor_velocity(motor_velocity.target, motor_velocity.factor);
    }
    if let Some(motor_position) = motor.position {
        builder = builder.motor_position(
            motor_position.target,
            motor_position.stiffness,
            motor_position.damping,
        );
    }
    if let Some(motor_max_force) = motor.max_force {
        builder = builder.motor_max_force(motor_max_force);
    }
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
                builder = builder.motor_model(axis, motor_model_from_rd(motor.model.clone()));
                if let Some(motor_velocity) = &motor.velocity {
                    builder = builder.motor_velocity(
                        axis,
                        motor_velocity.target,
                        motor_velocity.factor,
                    );
                }
                if let Some(motor_position) = &motor.position {
                    builder = builder.motor_position(
                        axis,
                        motor_position.target,
                        motor_position.stiffness,
                        motor_position.damping,
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

pub fn motor_model_from_rd(value: RdMotorModel) -> MotorModel {
    match value {
        RdMotorModel::AccelerationBased => MotorModel::AccelerationBased,
        RdMotorModel::ForceBased => MotorModel::ForceBased,
    }
}
