#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod avian_plugin;
mod controller_plugin;
mod conversions;
mod pre_processor_plugin;

pub use avian_plugin::RobocompAvianPlugin;
pub use controller_plugin::{ActiveJointControlTracker, RobocompAvianControllerPlugin};
pub use conversions::{
    angular_motor_from_rd, fixed_joint_from_rd, linear_motor_from_rd, motor_model_from_rd,
    prismatic_joint_from_rd, revolute_joint_from_rd, rigid_body_from_rc, spherical_joint_from_rd,
};
pub use controller_plugin::ControllerPluginSystemSet;
pub use pre_processor_plugin::PreProcessorPluginSystemSet;

pub use robocomp::*;
pub use robocomp::rc;
pub use robocomp::rd;
