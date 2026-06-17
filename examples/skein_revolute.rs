//! Example demonstrating a scene imported from Blender using Skein, with a revolute joint between two cubes.

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_skein::SkeinPlugin;
use robocomp::{RobocompPlugin, rc::RcSceneRoot};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        SkeinPlugin::default(), // Add Skein plugin for Blender scene import.
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || {
        info!("Robocomp skein revolute example running!")
    });

    // Run app...
    app.run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Skein Revolute Scene"),
        SceneRoot(asset_server.load(
            // Change this to your exported gltf file
            GltfAssetLabel::Scene(0).from_asset("revolute.gltf"),
        )),
        RcSceneRoot, // Add the RcSceneRoot component for Robocomp to recognize this as a scene root.
    ));
}
