use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    color::palettes::css::{GREEN, ORANGE, WHITE},
    light::light_consts::lux::CLEAR_SUNRISE,
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp::{
    RobocompPlugin,
    rc::{RcCollider, RcLinkRoot, RcRobotRoot, RcSceneRoot},
};

fn main() {
    let mut app = App::new();
    // Add Bevy, debug, camera plugins...
    app.add_plugins((
        DefaultPlugins,
        EguiPlugin::default(),
        WorldInspectorPlugin::default(),
        // Using the Bevy (built-in) free camera for now.
        // TODO: add custom orbit cam!
        FreeCameraPlugin,
    ));
    // Setup environment...
    app.insert_resource(GlobalAmbientLight {
        color: Color::from(WHITE),
        brightness: 500.,
        ..Default::default()
    });
    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || info!("Robocomp utils example running!"));

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera...
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        // This component stores all camera settings and state, which is used by the FreeCameraPlugin to
        // control it. These properties can be changed at runtime, but beware the controller system is
        // constantly using and modifying those values unless the enabled field is false.
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
    ));

    // Light...
    commands.spawn((
        Name::new("Sun Light"),
        DirectionalLight {
            color: Color::WHITE,
            illuminance: CLEAR_SUNRISE,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane...
    commands.spawn((
        Name::new("Ground Plane"),
        Mesh3d(meshes.add(Mesh::from(Plane3d::new(Vec3::Y, Vec2::splat(5.0))))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::from(GREEN),
            perceptual_roughness: 0.9,
            ..Default::default()
        })),
    ));

    // Scene...
    commands.spawn(scene(&mut meshes, &mut materials));
}

fn scene(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> impl Bundle {
    (
        Name::new("Sample Scene"),
        Transform::default(),
        Visibility::default(),
        RcSceneRoot, // Scene Root for Robocomp
        children![cube_object(meshes, materials)],
    )
}

fn cube_object(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> impl Bundle {
    (
        Name::new("Cube Object"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![(cube_link("Cube", meshes, materials))],
    )
}

fn cube_link(
    name: &'static str,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Bundle {
    (
        Name::new(format!("{} Link", name)),
        Transform::from_translation(Vec3::Y * 0.5),
        Visibility::default(),
        RcLinkRoot,
        RcCollider {
            keep_mesh: true,
            ..Default::default()
        },
        children![(
            Name::new(format!("{} Mesh", name)),
            Transform::default(),
            Visibility::default(),
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::from(ORANGE),
                perceptual_roughness: 0.5,
                ..Default::default()
            })),
        )],
    )
}
