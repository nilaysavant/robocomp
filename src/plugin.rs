use bevy::prelude::*;

use crate::{cleanup_manager::CleanupManagerPlugin, presets_plugin::PresetsPlugin};

use super::type_registry_plugin::TypeRegistryPlugin;

/// # Robocomp Plugin
///
/// Physics-agnostic robot compositor plugin for Bevy.
///
/// Registers marker components, Skein presets, and scene cleanup.
/// Add a physics backend plugin (e.g. [`robocomp_rapier::RobocompRapierPlugin`]) to spawn colliders and joints.
pub struct RobocompPlugin;

impl Plugin for RobocompPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CleanupManagerPlugin)
            .add_plugins(TypeRegistryPlugin)
            .add_plugins(PresetsPlugin)
            .add_systems(Startup, || info!("RobocompPlugin started..."));
    }
}
