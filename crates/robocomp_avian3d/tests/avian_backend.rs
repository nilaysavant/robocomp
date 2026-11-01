use avian3d::prelude::*;
use bevy::prelude::*;
use robocomp::rc::{RcCollider, RcLink};
use robocomp_avian3d::RobocompAvianPlugin;
use robocomp_test_utils::PhysicsTestBackend;

/// Avian physics backend for shared integration tests.
pub struct AvianBackend;

impl PhysicsTestBackend for AvianBackend {
    fn add_physics_plugins(app: &mut App) {
        app.add_plugins(RobocompAvianPlugin);
        app.add_plugins(RobocompAvianTestHarnessPlugin);
    }

    fn validate_colliders(world: &mut World) -> Result<()> {
        validate_colliders(world)
    }

    fn validate_links(world: &mut World) -> Result<()> {
        validate_links(world)
    }

    fn collider_proxy_type_path() -> &'static str {
        AvianColliderProxy::type_path()
    }
}

/// [`PhysicsPlugins`] with utilities for testing Robocomp scenes.
pub struct RobocompAvianTestHarnessPlugin;

impl Plugin for RobocompAvianTestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.register_type::<AvianColliderProxy>();
        app.add_observer(attach_collider_proxy);
        app.add_systems(Startup, || info!("RobocompAvianTestHarnessPlugin running!"));
    }
}

pub fn attach_collider_proxy(
    on_add: On<Add<Collider>>,
    colliders: Query<&Collider>,
    mut commands: Commands,
) -> Result {
    let entity = on_add.entity;
    let collider = colliders.get(entity)?;
    commands.entity(entity).insert(AvianColliderProxy {
        shape: format!("{collider:?}"),
    });
    Ok(())
}

#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct AvianColliderProxy {
    pub shape: String,
}

pub fn validate_colliders(world: &mut World) -> Result<()> {
    let mut colliders = world.query::<(&RcCollider, Option<&Collider>)>();
    let colliders_iter = colliders.iter(world);
    let count = colliders_iter.clone().count();
    assert!(count > 0, "No RcCollider components found in the world!");
    for (rc_collider, collider_opt) in colliders_iter {
        assert!(
            collider_opt.is_some(),
            "Collider not added for RcCollider: {:?}",
            rc_collider
        );
    }
    Ok(())
}

pub fn validate_links(world: &mut World) -> Result<()> {
    let mut links = world.query::<(
        Entity,
        &RcLink,
        Option<&RigidBody>,
        Option<&Collider>,
        Option<&Children>,
    )>();
    let mut colliders = world.query::<&Collider>();
    let links_iter = links.iter(world);
    let count = links_iter.clone().count();
    assert!(count > 0, "No RcLink components found in the world!");
    for (_entity, rc_link, rigid_body_opt, collider_opt, children_opt) in links_iter {
        assert!(
            rigid_body_opt.is_some(),
            "RigidBody not added for RcLink: {:?}",
            rc_link
        );
        if collider_opt.is_some() {
            continue;
        }
        if let Some(children) = children_opt {
            let mut found_collider = false;
            for child in children.iter() {
                if colliders.get(world, child).is_ok() {
                    found_collider = true;
                    break;
                }
            }
            assert!(
                found_collider,
                "No Collider found on RcLink {:?} or its children!",
                rc_link
            );
        } else {
            panic!(
                "No Collider found on RcLink {:?} and it has no children!",
                rc_link
            );
        }
    }
    Ok(())
}
