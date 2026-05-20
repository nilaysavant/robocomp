#![allow(clippy::type_complexity)]

use bevy::{mesh::MeshPlugin, prelude::*, scene::ScenePlugin};
use robocomp::{
    RobocompPlugin,
    rc::{RcSceneRoot, RcSceneRootIsProcessed, RcSceneRootReady},
};

/// Rapier physics utils for testing.
mod rapier;

pub use rapier::*;

/// # Bevy Testing Defaults Plugin
///
/// Provides a minimal Bevy setup for headless tests.
///
/// ## References
/// Bevy's official [`how_to_test_apps.rs`](https://github.com/bevyengine/bevy/blob/v0.18.0/tests/how_to_test_apps.rs).
/// Specifically [`create_test_app`](https://github.com/bevyengine/bevy/blob/5f8270f2e049f90139a503d1e930070d926f9427/tests/how_to_test_apps.rs#L52-L65).
pub struct BevyTestingDefaultsPlugin;

impl Plugin for BevyTestingDefaultsPlugin {
    fn build(&self, app: &mut App) {
        // Note the use of `MinimalPlugins` instead of `DefaultPlugins`.
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            MeshPlugin,
            ScenePlugin,
        ));
        // Inserting a `KeyCode` input resource allows us to inject keyboard inputs, as if the user had
        // pressed them.
        app.insert_resource(ButtonInput::<KeyCode>::default());
        // Spawning a fake window allows testing systems that require a window.
        app.world_mut().spawn(Window::default());
        // We need type registry for type reflection,
        // serialization and scene assertions to work!
        app.insert_resource(AppTypeRegistry::default());
        // Rest...
        app.add_systems(Startup, || info!("BevyTestingDefaultsPlugin running!"));
    }
}

/// # Robocomp Test Harness Plugin
///
/// [`RobocompPlugin`] with some additional type registrations and utilities for testing Robocomp scenes.
pub struct RobocompTestHarnessPlugin;

impl Plugin for RobocompTestHarnessPlugin {
    fn build(&self, app: &mut App) {
        // Add robocomp plugins...
        app.add_plugins(RobocompPlugin);
        // Type registrations...
        app.register_type::<Name>();
        app.register_type::<Children>();
        // Rest...
        app.add_systems(Startup, || info!("RobocompTestHarnessPlugin running!"));
    }
}

/// Runs [`App::update`] in a loop until the scene root is processed and ready.
///
/// Or until a `max_update_iterations` is reached, in which case an error is returned.
///
/// ## Returns
/// A vector of processed and ready scene root entities, if successful.
pub fn wait_for_scene_ready(app: &mut App, max_update_iterations: i32) -> Result<Vec<Entity>> {
    for i in 0..max_update_iterations {
        app.update();
        // Check if the scene root has been processed and is ready...
        let world = app.world_mut();
        let mut processed_scene_roots = world.query_filtered::<Entity, (
            With<RcSceneRoot>,
            With<RcSceneRootIsProcessed>,
            With<RcSceneRootReady>,
        )>();
        let processed_scene_roots = processed_scene_roots.iter(world).collect::<Vec<_>>();
        if !processed_scene_roots.is_empty() {
            println!("Scene root is processed and ready after {} updates!", i + 1);
            return Ok(processed_scene_roots);
        }
    }
    Err(format!(
        "Scene root was not processed and ready after {} updates!",
        max_update_iterations
    )
    .into())
}

/// Serialize the world's scene to a string for debugging and assertions in tests.
///
/// Ref: https://github.com/bevyengine/bevy/blob/v0.18.0/examples/scene/scene.rs
pub fn serialize_scene_world(world: &mut World) -> Result<String> {
    // We can now create our scene using DynamicScene or DynamicSceneBuilder.
    // For simplicity, we will create our scene using DynamicScene...
    let scene = DynamicScene::from_world(world);

    // Scenes can be serialized like this...
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry)?;

    Ok(serialized_scene)
}
