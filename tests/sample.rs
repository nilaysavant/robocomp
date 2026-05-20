use bevy::prelude::*;
use robocomp::rc::{BelongsToRcSceneRoot, RcCollider, RcLinkRoot, RcRobotRoot, RcSceneRoot};

use crate::common::{
    BevyTestingDefaultsPlugin, RapierColliderProxy, RobocompRapierTestHarnessPlugin,
    RobocompTestHarnessPlugin, serialize_scene_world, validate_colliders, validate_links,
    wait_for_scene_ready,
};

/// Common utilities for Robocomp tests.
mod common;

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(scene(&mut meshes));
}

fn scene(meshes: &mut Assets<Mesh>) -> impl Bundle {
    (
        Name::new("Sample Scene"),
        Transform::default(),
        Visibility::default(),
        RcSceneRoot, // Scene Root for Robocomp
        children![cube_object(meshes)],
    )
}

fn cube_object(meshes: &mut Assets<Mesh>) -> impl Bundle {
    (
        Name::new("Cube Object"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![(
            Name::new("Cube Link"),
            Transform::from_translation(Vec3::Y * 0.5),
            Visibility::default(),
            RcLinkRoot,
            RcCollider {
                keep_mesh: true,
                ..Default::default()
            },
            children![(
                Name::new("Cube Mesh"),
                Transform::default(),
                Visibility::default(),
                Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))))),
            )],
        )],
    )
}

#[test]
fn sample_test() -> Result {
    let mut app = App::new();
    // Plugins...
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    // Corresponding physics engine harness for this test...
    app.add_plugins(RobocompRapierTestHarnessPlugin);
    // Systems...
    app.add_systems(Startup, setup_scene);

    let scene_roots_ready = wait_for_scene_ready(&mut app, 5)?;

    let scene_root_ent = scene_roots_ready
        .first()
        .ok_or("No ready scene root entities found!")?;
    println!("Ready scene root: {:?}", scene_root_ent);

    let world = app.world_mut();
    let belongs_to_scene_root = world
        .query::<(Entity, &BelongsToRcSceneRoot)>()
        .iter(world)
        .filter(|(_, BelongsToRcSceneRoot(root))| *root == *scene_root_ent)
        .map(|(ent, _)| ent)
        .collect::<Vec<_>>();
    println!("Belonging to the scene root: {:?}", belongs_to_scene_root);

    // Validate...
    validate_colliders(world)?;
    validate_links(world)?;

    Ok(())
}

#[test]
fn scene_serialize_test() -> Result {
    // This test would involve serializing and deserializing the scene.
    // Implementation would depend on the specific serialization methods used in robocomp.
    let mut app = App::new();
    // Plugins...
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    // Corresponding physics engine harness for this test...
    app.add_plugins(RobocompRapierTestHarnessPlugin);
    // Systems...
    app.add_systems(Startup, setup_scene);

    wait_for_scene_ready(&mut app, 5)?;

    let serialized_scene = serialize_scene_world(app.world_mut())?;
    // Showing the scene in the console
    println!("{}", serialized_scene);

    assert!(serialized_scene.contains(RapierColliderProxy::type_path()));

    Ok(())
}
