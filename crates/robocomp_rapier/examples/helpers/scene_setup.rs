//! A helper module for setting up a basic scene with a camera, light, and ground plane.

use bevy::{
    color::palettes::css::WHITE, dev_tools::fps_overlay::FpsOverlayPlugin,
    light::light_consts::lux::CLEAR_SUNRISE, prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

/// # Scene Setup Plugin
///
/// A Bevy plugin that sets up a basic scene with a camera, light, and ground plane.
///
/// Not required for the Robocomp plugin to work, but useful for examples and testing.
pub struct SceneSetupPlugin;

impl Plugin for SceneSetupPlugin {
    fn build(&self, app: &mut App) {
        // Add Debug and Diagnostic plugins...
        app.add_plugins((
            FpsOverlayPlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ));
        // Setup environment...
        app.insert_resource(GlobalAmbientLight {
            color: Color::from(WHITE),
            brightness: 500.,
            ..Default::default()
        });
        app.insert_resource(ClearColor(Color::BLACK));

        // Systems...
        app.add_systems(Startup, setup_scene);

        // Rest...
        app.add_systems(Startup, || info!("SceneSetupPlugin running!"));
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera...
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::ONE * 5.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light...
    commands.spawn((
        Name::new("Sun Light"),
        DirectionalLight {
            illuminance: CLEAR_SUNRISE,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(1.0, 1.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane...
    commands.spawn((
        Name::new("Ground Plane"),
        Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(5.0))))),
        MeshMaterial3d(materials.add(StandardMaterial {
            perceptual_roughness: 0.,
            ..Default::default()
        })),
    ));
}
