use bevy::prelude::*;
use robocomp::RobocompPlugin;

use crate::pre_processor_plugin::PreProcessorPlugin;

/// # Robocomp Rapier Plugin
///
/// Rapier physics backend for the robot compositor.
///
/// Bundles [`RobocompPlugin`] with the Rapier pre-processor that spawns colliders and joints.
pub struct RobocompRapierPlugin;

impl Plugin for RobocompRapierPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RobocompPlugin, PreProcessorPlugin))
            .add_systems(Startup, || info!("RobocompRapierPlugin started..."));
    }
}
