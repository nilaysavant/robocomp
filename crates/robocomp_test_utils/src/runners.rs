use bevy::prelude::*;
use robocomp::rc::{BelongsToRcSceneRoot, RcRobotRoot};

use crate::{
    backend::PhysicsTestBackend,
    harness::{BevyTestingDefaultsPlugin, RobocompTestHarnessPlugin, wait_for_scene_ready},
    scenarios,
};

pub fn run_sample_scene_test<B: PhysicsTestBackend>() -> Result<()> {
    let mut app = App::new();
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    B::add_physics_plugins(&mut app);
    app.add_systems(Startup, setup_sample_scene);

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

    B::validate_colliders(world)?;
    B::validate_links(world)?;

    Ok(())
}

pub fn run_sample_scene_serialize_test<B: PhysicsTestBackend>() -> Result<()> {
    let mut app = App::new();
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    B::add_physics_plugins(&mut app);
    app.add_systems(Startup, setup_sample_scene);

    wait_for_scene_ready(&mut app, 5)?;

    let serialized_scene = crate::harness::serialize_scene_world(app.world_mut())?;
    println!("{}", serialized_scene);
    assert!(serialized_scene.contains(B::collider_proxy_type_path()));

    Ok(())
}

pub fn run_multi_robots_test<B: PhysicsTestBackend>() -> Result<()> {
    let mut app = App::new();
    app.add_plugins(BevyTestingDefaultsPlugin);
    app.add_plugins(RobocompTestHarnessPlugin);
    B::add_physics_plugins(&mut app);
    app.add_systems(Startup, setup_multi_robots_scene);

    let scene_roots_ready = wait_for_scene_ready(&mut app, 5)?;

    let serialized_scene = crate::harness::serialize_scene_world(app.world_mut())?;
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

    B::validate_colliders(world)?;
    B::validate_links(world)?;

    Ok(())
}

fn setup_sample_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(scenarios::sample_scene(&mut meshes));
}

fn setup_multi_robots_scene(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(scenarios::multi_robots_scene(&mut meshes));
}
