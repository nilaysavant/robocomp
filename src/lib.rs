#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

/// Cleanup manager plugin and related components/systems.
pub mod cleanup_manager;
/// Main plugin for the robot compositor.
pub mod plugin;
/// Plugins for Skein presets used in Blender.
mod presets_plugin;
/// Exposes the components for the robot composition/editing.
///
/// Used via external tools like Blender (via Skein) etc.
pub mod rc;
/// Describes the robot and its components.
pub mod rd;
/// Type registry plugin for the robot compositor.
mod type_registry_plugin;

pub use plugin::RobocompPlugin;
