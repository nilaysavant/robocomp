//! Example demonstrating a scene imported from Blender using Skein, with dual joints control.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;
use robocomp_avian3d::{
    RobocompAvianControllerPlugin, RobocompAvianPlugin, rc::RcSceneRoot,
};

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
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
        RobocompAvianPlugin,
        RobocompAvianControllerPlugin,
    ));

    app.add_systems(Startup, setup_scene);
    app.add_systems(Startup, || {
        info!("Robocomp skein dual joints control example running!")
    });
    app.run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Skein Dual Joints Control Scene"),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("dual_joints_ctrl.gltf"))),
        RcSceneRoot,
    ));
}
