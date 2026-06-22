use bevy::prelude::*;
use robocomp::RobocompPlugin;

use crate::pre_processor_plugin::PreProcessorPlugin;

/// # Robocomp Avian Plugin
///
/// Avian physics backend for the robot compositor.
///
/// Bundles [`RobocompPlugin`] with the Avian pre-processor that spawns colliders and joints.
pub struct RobocompAvianPlugin;

impl Plugin for RobocompAvianPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RobocompPlugin, PreProcessorPlugin))
            .add_systems(Startup, || info!("RobocompAvianPlugin started..."));
    }
}
