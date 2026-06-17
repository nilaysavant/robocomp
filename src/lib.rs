#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

/// Plugin for Robot Controlling.
mod controller_plugin;
/// Main plugin for the robot compositor.
pub mod plugin;
/// Marked entities pre-processor plugin.
mod pre_processor_plugin;
/// Plugins for Skein presets used in Blender.
mod presets_plugin;
/// Exposes the components for the robot composition/editing.
///
/// Used via external tools like Blenvy etc.
pub mod rc;
/// Describes the robot and its components.
pub mod rd;
/// Type registry plugin for the robot compositor.
mod type_registry_plugin;

/// Cleanup manager plugin and related components/systems.
mod cleanup_manager;

/// Exports the main plugin for the robot compositor.
pub use plugin::RobocompPlugin;

/// Exports the controller plugin for controlling robots.
pub use controller_plugin::RobocompControllerPlugin;
