use bevy::prelude::*;
use robocomp::rc::{RcCollider, RcLinkRoot, RcRobotRoot, RcSceneRoot};

/// Builds the sample scene used by integration tests.
pub fn sample_scene(meshes: &mut Assets<Mesh>) -> impl Bundle {
    (
        Name::new("Sample Scene"),
        Transform::default(),
        Visibility::default(),
        RcSceneRoot,
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

/// Builds the multi-robot scene used by integration tests.
pub fn multi_robots_scene(meshes: &mut Assets<Mesh>) -> impl Bundle {
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
        RcSceneRoot,
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
