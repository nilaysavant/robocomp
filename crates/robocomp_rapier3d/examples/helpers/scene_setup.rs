//! A helper module for setting up a basic scene with a camera, light, and ground plane.

use bevy::{
    color::palettes::css::WHITE,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    light::light_consts::lux::CLEAR_SUNRISE,
    prelude::*,
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
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    frame_time_graph_config: FrameTimeGraphConfig::target_fps(144.),
                    ..Default::default()
                },
            },
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
    let camera_transform =
        Transform::from_translation(Vec3::ONE * 8.).looking_at(Vec3::ZERO, Vec3::Y);
    let light_transform =
        Transform::from_translation(Vec3::new(1.0, 1.0, 0.0)).looking_at(Vec3::ZERO, Vec3::Y);
    let ground_mesh = meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(5.0))));
    let ground_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.,
        ..Default::default()
    });

    commands.spawn_scene_list(bsn_list! {
        Camera3d
        camera_transform
        --
        Name("Sun Light")
        DirectionalLight {
            illuminance: {CLEAR_SUNRISE},
            shadow_maps_enabled: true,
        }
        light_transform
        --
        Name("Ground Plane")
        Mesh3d({ground_mesh})
        MeshMaterial3d<StandardMaterial>({ground_material})
    });
}
