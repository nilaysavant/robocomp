use bevy::{mesh::MeshPlugin, prelude::*, scene::ScenePlugin};
use robocomp::rc::{RcSceneRoot, RcSceneRootIsProcessed, RcSceneRootReady};

/// Provides a minimal Bevy setup for headless tests.
pub struct BevyTestingDefaultsPlugin;

impl Plugin for BevyTestingDefaultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            MeshPlugin,
            ScenePlugin,
        ));
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.world_mut().spawn(Window::default());
        app.insert_resource(AppTypeRegistry::default());
        app.add_systems(Startup, || info!("BevyTestingDefaultsPlugin running!"));
    }
}

/// Extra type registrations used by integration tests.
pub struct RobocompTestHarnessPlugin;

impl Plugin for RobocompTestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Name>();
        app.register_type::<Children>();
        app.add_systems(Startup, || info!("RobocompTestHarnessPlugin running!"));
    }
}

/// Runs [`App::update`] until the scene root is processed and ready.
pub fn wait_for_scene_ready(app: &mut App, max_update_iterations: i32) -> Result<Vec<Entity>> {
    for i in 0..max_update_iterations {
        app.update();
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
pub fn serialize_scene_world(world: &mut World) -> Result<String> {
    let scene = DynamicScene::from_world(world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry)?;
    Ok(serialized_scene)
}
