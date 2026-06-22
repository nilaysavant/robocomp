//! Example demonstrating a scene imported from Blender using Skein, with a prismatic joint.

use bevy::prelude::*;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_skein::SkeinPlugin;
use robocomp_rapier3d::{RobocompRapierPlugin, rc::RcSceneRoot};

use crate::scene_setup::SceneSetupPlugin;
use crate::workspace_assets::workspace_assets_dir;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

#[path = "helpers/workspace_assets.rs"]
mod workspace_assets;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: workspace_assets_dir(),
            ..default()
        }),
        SceneSetupPlugin,
    ));

    app.add_plugins((
        SkeinPlugin::default(),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompRapierPlugin,
    ));

    app.add_systems(Startup, setup_scene);
    app.add_systems(Startup, || info!("Robocomp skein prismatic example running!"));
    app.run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Skein Prismatic Scene"),
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("prismatic.gltf"),
        )),
        RcSceneRoot,
    ));
}
