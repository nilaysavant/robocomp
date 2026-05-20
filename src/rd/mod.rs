//! URDF inspired robot description format.
//! ref: https://wiki.ros.org/urdf/XML
//! ref: https://articulatedrobotics.xyz/tutorials/ready-for-ros/urdf/

use std::{borrow::Cow, hash::Hash};

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_rapier3d::prelude::{
    FixedJointBuilder, JointAxis, MotorModel, PrismaticJointBuilder, RevoluteJointBuilder,
    SphericalJointBuilder,
};

/// Top Level Robot Descriptor and Manager.
///
/// - Used to describe/construct the robot and its components fully.
/// - Also to manage/interact with the robot and its components with a nice API.
///
/// ### References
/// - [URDF Robot](https://wiki.ros.org/urdf/XML/robot)
/// - [Articulated Robotics - Robot tag](https://articulatedrobotics.xyz/tutorials/ready-for-ros/urdf/#the-robot-tag-and-xml-declaration)
#[derive(Debug, Clone, Reflect, Component)]
pub struct RdRobot<Id: RdId> {
    pub links_by_name: HashMap<RdName, Id>,
    pub joints_by_name: HashMap<RdName, Id>,
}

impl<Id: RdId> RdRobot<Id> {
    pub fn insert_link(&mut self, name: RdName, id: Id) {
        self.links_by_name.insert(name, id);
    }

    pub fn insert_joint(&mut self, name: RdName, id: Id) {
        self.joints_by_name.insert(name, id);
    }

    pub fn remove_link(&mut self, name: &RdName) -> Option<Id> {
        self.links_by_name.remove(name)
    }

    pub fn remove_joint(&mut self, name: &RdName) -> Option<Id> {
        self.joints_by_name.remove(name)
    }

    pub fn get_link(&self, name: &RdName) -> Option<&Id> {
        self.links_by_name.get(name)
    }

    pub fn get_joint(&self, name: &RdName) -> Option<&Id> {
        self.joints_by_name.get(name)
    }
}

impl<Id: RdId> Default for RdRobot<Id> {
    fn default() -> Self {
        Self {
            links_by_name: HashMap::default(),
            joints_by_name: HashMap::default(),
        }
    }
}

/// `Id` trait bounds for Robot Descriptor Components.
///
/// ie. if `RdId` then it is: `Clone + Reflect + Eq + Hash`.
pub trait RdId: Clone + Reflect + Eq + Hash {}

/// Blanket Impl for `Id` trait bounds for Robot Descriptor Components.
///
/// ie. if `T` is `Clone + Reflect + Eq + Hash`, then `T` is also `RdId`.
impl<T: Clone + Reflect + Eq + Hash> RdId for T {}

/// Common name type for URDF inspired components.
#[derive(Debug, Clone, Reflect, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct RdName(pub Cow<'static, str>);

impl Default for RdName {
    fn default() -> Self {
        Self(Cow::from("unnamed"))
    }
}

impl RdName {
    pub fn new(name: &str) -> Self {
        Self(Cow::from(name.to_string()))
    }
}

impl From<Name> for RdName {
    fn from(name: Name) -> Self {
        Self(Cow::from(name.to_string()))
    }
}

impl From<RdName> for Name {
    fn from(value: RdName) -> Self {
        Self::new(value.0.to_string())
    }
}

/// Vector type for URDF inspired components.
pub type RdVector = Vec3;

/// Scaler type for URDF inspired components.
pub type RdScaler = f32;

/// Links which compose the robot.
///
/// ## References
/// - [URDF Link](https://wiki.ros.org/urdf/XML/link)
/// - [Articulated Robotics - Link tags](https://articulatedrobotics.xyz/tutorials/ready-for-ros/urdf/#link-tags)
#[derive(Debug, Clone, Reflect, Component)]
pub struct RdLink<Id: RdId> {
    /// Supports multiple colliders (as a group) for the link.
    pub colliders_by_name: HashMap<RdName, Id>,
}

impl<Id: RdId> RdLink<Id> {
    pub fn insert_collider(&mut self, name: RdName, id: Id) {
        self.colliders_by_name.insert(name.clone(), id.clone());
    }

    pub fn remove_collider(&mut self, name: &RdName) -> Option<Id> {
        self.colliders_by_name.remove(name)
    }

    pub fn get_collider(&self, name: &RdName) -> Option<&Id> {
        self.colliders_by_name.get(name)
    }
}

impl<Id: RdId> Default for RdLink<Id> {
    fn default() -> Self {
        Self {
            colliders_by_name: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Reflect, Component)]
pub struct RdCollider {
    /// Collider density.
    pub density: RdScaler,
}

impl Default for RdCollider {
    fn default() -> Self {
        Self { density: 1.0 }
    }
}

/// URDF inspired robot joints.
///
/// ## References
/// - [URDF Joint](https://wiki.ros.org/urdf/XML/joint)
/// - [Articulated Robotics - Joint tags](https://articulatedrobotics.xyz/tutorials/ready-for-ros/urdf/#joint-tags)
#[derive(Debug, Clone, Reflect, Component)]
pub struct RdJoint<Id: RdId> {
    /// Id to the parent link of the joint.
    pub parent: Id,
    /// Id to the child link of the joint.
    pub child: Id,
    pub kind: RdJointKind,
}

/// URDF inspired Joint Types. With associated data like anchors, axis, motors, limits etc.
#[derive(Debug, Clone, Reflect, Component)]
pub enum RdJointKind {
    Fixed(RdFixedJointData),
    Revolute(RdRevoluteJointData),
    Prismatic(RdPrismaticJointData),
    Spherical(RdSphericalJointData),
}

#[derive(Debug, Clone, Reflect)]
pub struct RdFixedJointData {
    pub local_anchor1: RdVector,
    pub local_anchor2: RdVector,
}

impl From<RdFixedJointData> for FixedJointBuilder {
    fn from(value: RdFixedJointData) -> Self {
        let RdFixedJointData {
            local_anchor1,
            local_anchor2,
        } = value;

        FixedJointBuilder::new()
            .local_anchor1(local_anchor1)
            .local_anchor2(local_anchor2)
    }
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct RdRevoluteJointData {
    pub axis: RdVector,
    pub local_anchor1: RdVector,
    pub local_anchor2: RdVector,
    pub motor: RdMotor,
    pub limits: Option<RdJointLimits>,
}

impl From<RdRevoluteJointData> for RevoluteJointBuilder {
    fn from(value: RdRevoluteJointData) -> Self {
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
            .motor_model(motor.model.into());
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
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct RdPrismaticJointData {
    pub axis: RdVector,
    pub local_anchor1: RdVector,
    pub local_anchor2: RdVector,
    pub motor: RdMotor,
    pub limits: Option<RdJointLimits>,
}

impl From<RdPrismaticJointData> for PrismaticJointBuilder {
    fn from(value: RdPrismaticJointData) -> Self {
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
            .motor_model(motor.model.into());
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
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct RdSphericalJointData {
    pub local_anchor1: RdVector,
    pub local_anchor2: RdVector,
    pub motor_x: Option<RdMotor>,
    pub motor_y: Option<RdMotor>,
    pub motor_z: Option<RdMotor>,
    pub limit_ang_x: Option<RdJointLimits>,
    pub limit_ang_y: Option<RdJointLimits>,
    pub limit_ang_z: Option<RdJointLimits>,
}

impl From<RdSphericalJointData> for SphericalJointBuilder {
    fn from(value: RdSphericalJointData) -> Self {
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
        // Set motors...
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
                    builder = builder.motor_model(axis, motor.model.clone().into());
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
        // Set limits...
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
}

#[derive(Debug, Clone, Reflect, Default)]
pub struct RdMotor {
    pub model: RdMotorModel,
    pub velocity: Option<RdMotorVelocity>,
    pub position: Option<RdMotorPosition>,
    pub max_force: Option<RdScaler>,
}

#[derive(Debug, Clone, Reflect, Default)]
pub enum RdMotorModel {
    AccelerationBased,
    #[default]
    ForceBased,
}

impl From<RdMotorModel> for MotorModel {
    fn from(value: RdMotorModel) -> Self {
        match value {
            RdMotorModel::AccelerationBased => MotorModel::AccelerationBased,
            RdMotorModel::ForceBased => MotorModel::ForceBased,
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct RdMotorVelocity {
    pub target: RdScaler,
    pub factor: RdScaler,
}

#[derive(Debug, Clone, Reflect)]
pub struct RdMotorPosition {
    pub target: RdScaler,
    pub stiffness: RdScaler,
    pub damping: RdScaler,
}

#[derive(Debug, Clone, Reflect)]
pub struct RdJointLimits {
    pub min: RdScaler,
    pub max: RdScaler,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn change_robot_joint_motor_velocity() {
        let mut app = App::new();

        app.add_systems(Update, update_robot_joint_motor_velocity);

        let mut commands = app.world_mut().commands();
        // Robot...
        let mut rd_robot = RdRobot::<Entity>::default();
        // Spawn links...
        // Link 1...
        let mut link_1_rd_link = RdLink::<Entity>::default();
        let link_1_collider_name = RdName::new("link_1_collider");
        let link_1_collider = commands
            .spawn((
                Name::from(link_1_collider_name.clone()),
                RdCollider::default(),
            ))
            .id();
        link_1_rd_link.insert_collider(link_1_collider_name.clone(), link_1_collider);
        let link_1_name = RdName::new("link_1");
        let link_1 = commands
            .spawn((Name::from(link_1_name.clone()), link_1_rd_link))
            .id();
        rd_robot.insert_link(link_1_name.clone(), link_1);
        // Link 2...
        let mut link_2_rd_link = RdLink::<Entity>::default();
        let link_2_collider_name = RdName::new("link_2_collider");
        let link_2_collider = commands
            .spawn((
                Name::from(link_2_collider_name.clone()),
                RdCollider::default(),
            ))
            .id();
        link_2_rd_link.insert_collider(link_2_collider_name.clone(), link_2_collider);
        let link_2_name = RdName::new("link_2");
        let link_2 = commands
            .spawn((Name::from(link_2_name.clone()), link_2_rd_link))
            .id();
        rd_robot.insert_link(link_2_name.clone(), link_2);
        // Spawn joints...
        // Joint 1...
        let joint_1_rd_joint = RdJoint {
            parent: link_1,
            child: link_2,
            kind: RdJointKind::Revolute(RdRevoluteJointData {
                axis: Vec3::Y,
                local_anchor1: Vec3::Y / 2.,
                local_anchor2: Vec3::Y / 2.,
                motor: RdMotor {
                    velocity: Some(RdMotorVelocity {
                        target: 0.0,
                        factor: 1.0,
                    }),
                    ..Default::default()
                },
                limits: None,
            }),
        };
        let joint_1_name = RdName::new("joint_1");
        let joint_1 = commands
            .spawn((Name::from(joint_1_name.clone()), joint_1_rd_joint))
            .id();
        rd_robot.insert_joint(joint_1_name.clone(), joint_1);
        // Spawn robot...
        let robot_name = RdName::new("robot");
        commands.spawn((Name::from(robot_name), rd_robot));

        // System to update robot joint motor velocity...
        pub fn update_robot_joint_motor_velocity(
            robots: Query<&RdRobot<Entity>>,
            mut joints: Query<&mut RdJoint<Entity>>,
        ) {
            for robot in robots.iter() {
                for mut joint in joints.iter_mut() {
                    if joint.parent == *robot.get_link(&RdName::new("link_1")).unwrap()
                        && joint.child == *robot.get_link(&RdName::new("link_2")).unwrap()
                    {
                        if let RdJointKind::Revolute(ref mut data) = joint.kind {
                            data.motor.velocity = Some(RdMotorVelocity {
                                target: 10.0,
                                factor: 1.0,
                            });
                        }
                    }
                }
            }
        }

        // Run systems...
        app.update();

        // Check if the motor velocity was updated...
        let joint = app.world().get::<RdJoint<Entity>>(joint_1).unwrap();
        if let RdJointKind::Revolute(data) = &joint.kind {
            if let Some(RdMotorVelocity { target, factor }) = data.motor.velocity {
                assert_eq!(target, 10.0);
                assert_eq!(factor, 1.0);
            } else {
                panic!("Motor velocity is not set!");
            }
        } else {
            panic!("Joint kind is not Revolute!");
        }
    }
}
