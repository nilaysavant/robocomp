use bevy::prelude::*;
use bevy_rapier3d::{parry::shape::TypedShape, prelude::*};
use robocomp::rc::{RcCollider, RcLink};
use robocomp_rapier3d::RobocompRapierPlugin;
use robocomp_test_utils::PhysicsTestBackend;

/// Rapier physics backend for shared integration tests.
pub struct RapierBackend;

impl PhysicsTestBackend for RapierBackend {
    fn add_physics_plugins(app: &mut App) {
        app.add_plugins(RobocompRapierPlugin);
        app.add_plugins(RobocompRapierTestHarnessPlugin);
    }

    fn validate_colliders(world: &mut World) -> Result<()> {
        validate_colliders(world)
    }

    fn validate_links(world: &mut World) -> Result<()> {
        validate_links(world)
    }

    fn collider_proxy_type_path() -> &'static str {
        RapierColliderProxy::type_path()
    }
}

/// [`RapierPhysicsPlugin`] with utilities for testing Robocomp scenes.
pub struct RobocompRapierTestHarnessPlugin;

impl Plugin for RobocompRapierTestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.register_type::<RapierColliderProxy>();
        app.register_type::<RapierTypedShapeProxy>();
        app.add_observer(attach_collider_proxy);
        app.add_systems(Startup, || info!("RobocompRapierTestHarnessPlugin running!"));
    }
}

pub fn attach_collider_proxy(
    on_add: On<Add<Collider>>,
    colliders: Query<&Collider>,
    mut commands: Commands,
) -> Result {
    let entity = on_add.entity;
    let collider = colliders.get(entity)?;
    println!(
        "Attaching RapierColliderProxy to entity {:?} with collider: {:?}",
        entity, collider
    );
    commands
        .entity(entity)
        .insert(RapierColliderProxy::from(collider.clone()));
    Ok(())
}

#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RapierColliderProxy {
    #[reflect(ignore)]
    pub _collider: Collider,
    pub scale: Vec3,
    pub shape: RapierTypedShapeProxy,
}

impl From<Collider> for RapierColliderProxy {
    fn from(collider: Collider) -> Self {
        let scale = collider.scale();
        let shape = RapierTypedShapeProxy::from(collider.as_typed_shape().as_typed_shape());
        Self {
            _collider: collider,
            scale,
            shape,
        }
    }
}

#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub enum RapierTypedShapeProxy {
    Ball,
    Cuboid,
    Capsule,
    Segment,
    Triangle,
    Voxels,
    TriMesh,
    Polyline,
    HalfSpace,
    HeightField,
    Compound,
    ConvexPolyhedron,
    Cylinder,
    Cone,
    RoundCuboid,
    RoundTriangle,
    RoundCylinder,
    RoundCone,
    RoundConvexPolyhedron,
    Custom,
}

impl From<TypedShape<'_>> for RapierTypedShapeProxy {
    fn from(typed_shape: TypedShape) -> Self {
        match typed_shape {
            TypedShape::Ball(_) => RapierTypedShapeProxy::Ball,
            TypedShape::Cuboid(_) => RapierTypedShapeProxy::Cuboid,
            TypedShape::Capsule(_) => RapierTypedShapeProxy::Capsule,
            TypedShape::Segment(_) => RapierTypedShapeProxy::Segment,
            TypedShape::Triangle(_) => RapierTypedShapeProxy::Triangle,
            TypedShape::Voxels(_) => RapierTypedShapeProxy::Voxels,
            TypedShape::TriMesh(_) => RapierTypedShapeProxy::TriMesh,
            TypedShape::Polyline(_) => RapierTypedShapeProxy::Polyline,
            TypedShape::HalfSpace(_) => RapierTypedShapeProxy::HalfSpace,
            TypedShape::HeightField(_) => RapierTypedShapeProxy::HeightField,
            TypedShape::Compound(_) => RapierTypedShapeProxy::Compound,
            TypedShape::ConvexPolyhedron(_) => RapierTypedShapeProxy::ConvexPolyhedron,
            TypedShape::Cylinder(_) => RapierTypedShapeProxy::Cylinder,
            TypedShape::Cone(_) => RapierTypedShapeProxy::Cone,
            TypedShape::RoundCuboid(_) => RapierTypedShapeProxy::RoundCuboid,
            TypedShape::RoundTriangle(_) => RapierTypedShapeProxy::RoundTriangle,
            TypedShape::RoundCylinder(_) => RapierTypedShapeProxy::RoundCylinder,
            TypedShape::RoundCone(_) => RapierTypedShapeProxy::RoundCone,
            TypedShape::RoundConvexPolyhedron(_) => RapierTypedShapeProxy::RoundConvexPolyhedron,
            TypedShape::Custom(_) => RapierTypedShapeProxy::Custom,
        }
    }
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
