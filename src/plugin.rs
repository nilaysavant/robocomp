use bevy::prelude::*;

use crate::{cleanup_manager::CleanupManagerPlugin, presets_plugin::PresetsPlugin};

use super::{pre_processor_plugin::PreProcessorPlugin, type_registry_plugin::TypeRegistryPlugin};

/// # Robocomp Plugin
///
/// Robot compositor plugin for Bevy.
pub struct RobocompPlugin;

impl Plugin for RobocompPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .add_plugins(CleanupManagerPlugin)
            // Register all types for the robot compositor.
            .add_plugins(TypeRegistryPlugin)
            // Add the Presets plugin for Skein presets used in Blender.
            .add_plugins(PresetsPlugin)
            // Process the added entities with marked components...
            .add_plugins(PreProcessorPlugin)
            // Systems...
            .add_systems(Startup, || info!("RobocompPlugin started..."));
    }
}
