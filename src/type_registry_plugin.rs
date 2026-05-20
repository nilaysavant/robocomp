use bevy::prelude::*;

use crate::controller_plugin::ActiveJointControlTracker;

pub use super::rc::*;
pub use super::rd::*;

pub struct TypeRegistryPlugin;

impl Plugin for TypeRegistryPlugin {
    fn build(&self, app: &mut App) {
        app // App
            // Descriptor types...
            // Top level...
            .register_type::<RdRobot<Entity>>()
            .register_type::<RdName>()
            .register_type::<RdVector>()
            .register_type::<RdScaler>()
            // Joint types...
            .register_type::<RdJoint<Entity>>()
            .register_type::<RdJointKind>()
            .register_type::<RdFixedJointData>()
            .register_type::<RdRevoluteJointData>()
            .register_type::<RdPrismaticJointData>()
            .register_type::<RdSphericalJointData>()
            .register_type::<RdMotor>()
            .register_type::<RdMotorModel>()
            .register_type::<RdMotorVelocity>()
            .register_type::<RdMotorPosition>()
            .register_type::<RdJointLimits>()
            // Link types...
            .register_type::<RdLink<Entity>>()
            .register_type::<RdCollider>()
            .register_type::<RcColliderConstruction>()
            // Exposed Components...
            // Core Components...
            .register_type::<RcSceneRoot>()
            .register_type::<BelongsToRcSceneRoot>()
            .register_type::<RcGroup>()
            .register_type::<BelongsToRcGroup>()
            .register_type::<RcRobotRoot>()
            .register_type::<RcRobotGraph>()
            .register_type::<RcRobotGraphLink>()
            .register_type::<RcRobotGraphJoint>()
            .register_type::<BelongsToRcRobotRoot>()
            .register_type::<RcLink>()
            .register_type::<BelongsToRcLink>()
            .register_type::<RcLinkRoot>()
            .register_type::<RcRigidBody>()
            .register_type::<RcCcd>()
            .register_type::<RcDisableSleep>()
            .register_type::<RcCollider>()
            .register_type::<RcJoint>()
            .register_type::<RcJointCategory>()
            .register_type::<RcJointKind>()
            .register_type::<RcJointMotorControlConfig>()
            .register_type::<RcRevoluteJointMotor>()
            .register_type::<RcRevoluteJointLimits>()
            .register_type::<RcPrismaticJointMotor>()
            .register_type::<RcPrismaticJointLimits>()
            .register_type::<RcSphericalJointMotor>()
            .register_type::<RcSphericalJointLimits>()
            .register_type::<RcJointLocalAnchor1>()
            .register_type::<RcJointLocalAnchor2>()
            // Ik Components...
            .register_type::<RcIkTarget>()
            .register_type::<RcIkTargetSource>()
            .register_type::<RcIkAffectedLinks>()
            // Controller components...
            .register_type::<ActiveJointControlTracker>()
            // Rest...
            .add_systems(Startup, || info!("TypeRegistryPlugin started..."));
    }
}
