use bevy::prelude::*;
use bevy_rapier3d::{parry::shape::TypedShape, prelude::*};
use robocomp::rc::{RcCollider, RcLink};

/// # Robocomp Rapier Test Harness Plugin
///
/// [`RapierPhysicsPlugin`] with some additional type registrations and utilities for testing Robocomp scenes.
pub struct RobocompRapierTestHarnessPlugin;

impl Plugin for RobocompRapierTestHarnessPlugin {
    fn build(&self, app: &mut App) {
        // Add robocomp plugins...
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        // Type registrations...
        app.register_type::<RapierColliderProxy>();
        app.register_type::<RapierTypedShapeProxy>();
        // Systems and Observers...
        app.add_observer(attach_collider_proxy);
        // Rest...
        app.add_systems(Startup, || {
            info!("RobocompRapierTestHarnessPlugin running!")
        });
    }
}

/// Adds a [`RapierColliderProxy`] to entities when [`Collider`] is added.
pub fn attach_collider_proxy(
    on_add: On<Add, Collider>,
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

/// Proxy component for [`Collider`] for reflection in tests.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RapierColliderProxy {
    #[reflect(ignore)]
    pub collider: Collider,
    pub scale: Vec3,
    pub shape: RapierTypedShapeProxy,
}

impl From<Collider> for RapierColliderProxy {
    fn from(collider: Collider) -> Self {
        let scale = collider.scale();
        let shape = RapierTypedShapeProxy::from(collider.as_typed_shape().as_typed_shape());
        Self {
            collider,
            scale,
            shape,
        }
    }
}

/// Proxy component for [`TypedShape`] for reflection in tests.
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
        // Match on the specific shape type and create corresponding proxy variant.
        // This is a simplified example, you would need to implement all shape types you want to support.
        match typed_shape {
            TypedShape::Ball(_ball) => RapierTypedShapeProxy::Ball,
            TypedShape::Cuboid(_cuboid) => RapierTypedShapeProxy::Cuboid,
            TypedShape::Capsule(_capsule) => RapierTypedShapeProxy::Capsule,
            TypedShape::Segment(_segment) => RapierTypedShapeProxy::Segment,
            TypedShape::Triangle(_triangle) => RapierTypedShapeProxy::Triangle,
            TypedShape::Voxels(_voxels) => RapierTypedShapeProxy::Voxels,
            TypedShape::TriMesh(_tri_mesh) => RapierTypedShapeProxy::TriMesh,
            TypedShape::Polyline(_polyline) => RapierTypedShapeProxy::Polyline,
            TypedShape::HalfSpace(_half_space) => RapierTypedShapeProxy::HalfSpace,
            TypedShape::HeightField(_height_field) => RapierTypedShapeProxy::HeightField,
            TypedShape::Compound(_compound) => RapierTypedShapeProxy::Compound,
            TypedShape::ConvexPolyhedron(_convex_polyhedron) => {
                RapierTypedShapeProxy::ConvexPolyhedron
            }
            TypedShape::Cylinder(_cylinder) => RapierTypedShapeProxy::Cylinder,
            TypedShape::Cone(_cone) => RapierTypedShapeProxy::Cone,
            TypedShape::RoundCuboid(_round_cuboid) => RapierTypedShapeProxy::RoundCuboid,
            TypedShape::RoundTriangle(_round_triangle) => RapierTypedShapeProxy::RoundTriangle,
            TypedShape::RoundCylinder(_round_cylinder) => RapierTypedShapeProxy::RoundCylinder,
            TypedShape::RoundCone(_round_cone) => RapierTypedShapeProxy::RoundCone,
            TypedShape::RoundConvexPolyhedron(_round_convex_polyhedron) => {
                RapierTypedShapeProxy::RoundConvexPolyhedron
            }
            TypedShape::Custom(_custom) => RapierTypedShapeProxy::Custom,
        }
    }
}

/// Validates that all [`RcCollider`] components in the world have corresponding `Collider` components added.
pub fn validate_colliders(world: &mut World) -> Result {
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

/// Validates the all [`RcLink`]s in the world have corresponding [`RigidBody`] components added.
///
/// With at least one [`Collider`] on itself or on its children.
pub fn validate_links(world: &mut World) -> Result {
    let mut links = world.query::<(
        Entity,
        &RcLink,
        Option<&RigidBody>,
        Option<&Collider>,
        Option<&Children>,
    )>();
    let mut colliders = world.query::<(&Collider)>();
    let links_iter = links.iter(world);
    let count = links_iter.clone().count();
    assert!(count > 0, "No RcLink components found in the world!");
    for (_entity, rc_link, rigid_body_opt, collider_opt, children_opt) in links_iter {
        assert!(
            rigid_body_opt.is_some(),
            "RigidBody not added for RcLink: {:?}",
            rc_link
        );
        // Check for colliders on self or children...
        if collider_opt.is_some() {
            // Collider found on self, all good.
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
