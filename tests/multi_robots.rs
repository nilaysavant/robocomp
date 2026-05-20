use bevy::prelude::*;
use robocomp::rc::{RcCollider, RcLinkRoot, RcRobotRoot, RcSceneRoot};

use crate::common::{
    BevyTestingDefaultsPlugin, RobocompRapierTestHarnessPlugin, RobocompTestHarnessPlugin,
    serialize_scene_world, validate_colliders, validate_links, wait_for_scene_ready,
};

/// Common utilities for Robocomp tests.
mod common;

fn setup_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(scene(&mut meshes));
}

fn scene(meshes: &mut Assets<Mesh>) -> impl Bundle {
    let link_mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.5))));
    let robots = (0..3).map(move |i| {
        robot(
            link_mesh.clone(),
            format!("Robot {}", i),
            Transform::from_translation(Vec3::X * i as f32 * 2.0),
        )
    });

    (
        Name::new("Multi Robots Scene"),
        RcSceneRoot, // Scene Root for Robocomp
        Transform::default(),
        Visibility::default(),
        Children::spawn(SpawnIter(robots)),
    )
}

fn robot(link_mesh: Handle<Mesh>, name: String, transform: Transform) -> impl Bundle {
    let robot_name = name.clone();
    let robot_links = (1..=3).map(move |i| {
        robot_link(
            link_mesh.clone(),
            Transform::from_translation(Vec3::Y * 0.5 * i as f32),
            format!("{} Link {}", robot_name.clone(), i),
        )
    });
    (
        Name::new(name.clone()),
        RcRobotRoot,
        transform,
        Visibility::default(),
        Children::spawn(SpawnIter(robot_links)),
    )
}

fn robot_link(mesh_hdl: Handle<Mesh>, transform: Transform, name: String) -> impl Bundle {
    (
        Name::new(name.clone()),
        transform,
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
            Mesh3d(mesh_hdl),
        )],
    )
}

#[test]
fn multi_robots() -> Result {
    let mut app = App::new();
    // Plugins...
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    // Corresponding physics engine harness for this test...
    app.add_plugins(RobocompRapierTestHarnessPlugin);
    // Systems...
    app.add_systems(Startup, setup_scene);

    let scene_roots_ready = wait_for_scene_ready(&mut app, 5)?;

    let serialized_scene = serialize_scene_world(app.world_mut())?;
    println!("Serialized scene:\n{}", serialized_scene);
    assert!(
        serialized_scene.contains("Multi Robots Scene"),
        "Serialized scene should contain 'Multi Robots Scene'"
    );

    let scene_root_ent = scene_roots_ready
        .first()
        .ok_or("No ready scene root entities found!")?;
    println!("Ready scene root: {:?}", scene_root_ent);

    let world = app.world_mut();
    let robots = world
        .query_filtered::<Entity, With<RcRobotRoot>>()
        .iter(world)
        .collect::<Vec<_>>();
    println!("Robots found: {:?}", robots);
    assert!(
        robots.len() == 3,
        "There should be 3 robots, found {}",
        robots.len()
    );

    // Validate...
    validate_colliders(world)?;
    validate_links(world)?;

    Ok(())
}
