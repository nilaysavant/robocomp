//! A simple rigid bodies example with a fixed table and cube(s) that fall onto it.

use bevy::{
    color::palettes::tailwind::{BLUE_400, SLATE_500},
    prelude::*,
};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use robocomp_rapier3d::{
    RobocompRapierPlugin,
    rc::{RcCollider, RcDisableSleep, RcLink, RcLinkRoot, RcRigidBody, RcRobotRoot, RcSceneRoot},
    rd::RdName,
};

use crate::scene_setup::SceneSetupPlugin;

#[path = "helpers/scene_setup.rs"]
mod scene_setup;

/// Wide, flat cuboid used as a fixed table surface.
const TABLE_SIZE: Vec3 = Vec3::new(5.0, 0.2, 5.0);
const CUBE_SIZE: f32 = 1.0;
/// Gap between the table top and the falling cube's center, before half-extent offset.
const FALL_HEIGHT: f32 = 2.5;

fn main() {
    let mut app = App::new();
    // Add Bevy default and basic scene setup plugins...
    app.add_plugins((DefaultPlugins, SceneSetupPlugin));

    // Add rapier physics and robocomp plugins...
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        RobocompRapierPlugin,
    ));

    // Systems...
    app.add_systems(Startup, setup_scene);

    // Rest...
    app.add_systems(Startup, || {
        info!("Robocomp simple rigid bodies example running!")
    });

    // Run app...
    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scene...
    commands
        .spawn((
            Name::new("Simple Rigid Bodies Scene"),
            Transform::default(),
            Visibility::default(),
            RcSceneRoot, // Scene Root for Robocomp
        ))
        .with_child(rigid_bodies_robot(&mut meshes, &mut materials));
}

fn rigid_bodies_robot(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Bundle {
    let table_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(TABLE_SIZE)));
    let table_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(SLATE_500),
        perceptual_roughness: 0.8,
        ..Default::default()
    });

    let cube_mesh_hdl = meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(CUBE_SIZE))));
    let cube_mat_hdl = materials.add(StandardMaterial {
        base_color: Color::from(BLUE_400),
        perceptual_roughness: 0.,
        ..Default::default()
    });

    let table_top = TABLE_SIZE.y;
    let cube_half = CUBE_SIZE / 2.0;

    (
        Name::new("Rigid Bodies"),
        RcRobotRoot,
        Transform::default(),
        Visibility::default(),
        children![
            (
                link("Table", RcRigidBody::Fixed, table_mesh_hdl, table_mat_hdl,),
                RcLinkRoot, // Mark the first link as the root link of the robot.
                Transform::from_translation(Vec3::new(0., TABLE_SIZE.y / 2., 0.)),
            ),
            (
                link(
                    "Falling Cube",
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone(),
                ),
                Transform::from_translation(Vec3::new(
                    -1.,
                    table_top + FALL_HEIGHT + cube_half,
                    0.
                )),
                RcDisableSleep, // Keep the dynamic link awake.
            ),
            (
                link_multi_colliders(
                    "Falling Cube Multi Colliders",
                    RcRigidBody::Dynamic,
                    cube_mesh_hdl.clone(),
                    cube_mat_hdl.clone(),
                ),
                Transform::from_translation(Vec3::new(1., table_top + FALL_HEIGHT + cube_half, 0.)),
                RcDisableSleep, // Keep the dynamic link awake.
            ),
        ],
    )
}

/// Link with a single child mesh that has a collider.
fn link(
    name: &'static str,
    rigid_body: RcRigidBody,
    mesh_hdl: Handle<Mesh>,
    material_hdl: Handle<StandardMaterial>,
) -> impl Bundle {
    (
        Name::new(format!("{} Link", name)),
        RcLink {
            name: RdName::new(name),
            rigid_body,
        },
        Visibility::default(),
        RcCollider {
            keep_mesh: true,
            ..Default::default()
        },
        children![(
            Name::new(format!("{} Mesh", name)),
            Transform::default(),
            Visibility::default(),
            Mesh3d(mesh_hdl),
            MeshMaterial3d(material_hdl),
        )],
    )
}

/// Link with multiple child meshes that have colliders.
fn link_multi_colliders(
    name: &'static str,
    rigid_body: RcRigidBody,
    mesh_hdl: Handle<Mesh>,
    material_hdl: Handle<StandardMaterial>,
) -> impl Bundle {
    (
        Name::new(format!("{} Link", name)),
        RcLink {
            name: RdName::new(name),
            rigid_body,
        },
        Visibility::default(),
        children![
            (
                Name::new(format!("{} Mesh Node 1", name)),
                Transform::from_xyz(0., 0.0, -0.6),
                Visibility::default(),
                RcCollider {
                    keep_mesh: true,
                    ..Default::default()
                },
                children![(
                    Name::new(format!("{} Mesh Node 1 Mesh", name)),
                    Transform::default(),
                    Visibility::default(),
                    Mesh3d(mesh_hdl.clone()),
                    MeshMaterial3d(material_hdl.clone()),
                )],
            ),
            (
                Name::new(format!("{} Mesh Node 2", name)),
                Transform::from_xyz(0., 0.0, 0.6),
                Visibility::default(),
                RcCollider {
                    keep_mesh: true,
                    ..Default::default()
                },
                children![(
                    Name::new(format!("{} Mesh Node 2 Mesh", name)),
                    Transform::default(),
                    Visibility::default(),
                    Mesh3d(mesh_hdl.clone()),
                    MeshMaterial3d(material_hdl.clone()),
                )],
            )
        ],
    )
}
