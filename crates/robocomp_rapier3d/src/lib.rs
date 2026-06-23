#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![doc = include_str!("../README.md")]

mod controller_plugin;
mod conversions;
mod pre_processor_plugin;
mod rapier_plugin;

pub use controller_plugin::{ActiveJointControlTracker, RobocompRapierControllerPlugin};
pub use pre_processor_plugin::PreProcessorPluginSystemSet;
pub use conversions::{
    fixed_joint_builder_from_rd, motor_model_from_rd, prismatic_joint_builder_from_rd,
    rapier_motor_model_from_rd, revolute_joint_builder_from_rd, rigid_body_from_rc,
    spherical_joint_builder_from_rd, stiffness_damping_from_rd,
};
pub use rapier_plugin::RobocompRapierPlugin;

pub use robocomp::*;
pub use robocomp::rc;
pub use robocomp::rd;
