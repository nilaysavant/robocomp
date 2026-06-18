use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::conversions;
use robocomp::cleanup_manager::Cleanup;

use robocomp::rc::{
    BelongsToRcGroup, BelongsToRcLink, BelongsToRcRobotRoot, BelongsToRcSceneRoot, RcCcd,
    RcCollider, RcColliderConstruction, RcColliderIsProcessed, RcDisableSleep, RcGroup,
    RcGroupIsProcessed, RcJoint, RcJointCategory, RcJointIsProcessed, RcJointKind,
    RcJointLocalAnchor1, RcJointLocalAnchor2, RcJointMotorControlConfig, RcLink, RcLinkIsProcessed,
    RcPrismaticJointLimits, RcPrismaticJointMotor, RcRevoluteJointLimits, RcRevoluteJointMotor,
    RcRobotGraph, RcRobotGraphJoint, RcRobotGraphLink, RcRobotRoot, RcRobotRootIsProcessed,
    RcSceneRoot, RcSceneRootIsProcessed, RcSceneRootReady, RcSceneRootWRobotRootsDetached,
    RcSphericalJointLimits, RcSphericalJointMotor,
};

/// Processes marked entities with compositor components.
pub struct PreProcessorPlugin;

impl Plugin for PreProcessorPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .configure_sets(Update, PreProcessorPluginSystemSet)
            .add_systems(
                Update,
                (
                    process_group_markers,
                    process_scene_root_markers,
                    process_root_markers,
                    process_collider_markers,
                    process_link_markers,
                    process_joint_markers,
                    check_processing_complete,
                    detach_root_markers_and_flatten_hierarchy,
                )
                    .chain()
                    .in_set(PreProcessorPluginSystemSet),
            )
            // Rest...
            .add_systems(Startup, || info!("PreProcessorPlugin started..."));
    }
}

/// System set for [`PreProcessorPlugin`]'s systems.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct PreProcessorPluginSystemSet;

/// Process [`RcGroup`] when added.
pub fn process_group_markers(
    group_markers: Query<Entity, (With<RcGroup>, Without<RcGroupIsProcessed>, Without<Cleanup>)>,
    children: Query<&Children, Without<Cleanup>>,
    mut commands: Commands,
) {
    for group_ent in group_markers.iter() {
        debug!("Processing RcGroup: {:?}", group_ent);
        let num_of_recursive_children = children.iter_descendants(group_ent).count();
        if num_of_recursive_children == 0 {
            continue;
        }
        // Mark all children as belonging to this group...
        for child_ent in children.iter_descendants(group_ent) {
            commands
                .entity(child_ent)
                .insert(BelongsToRcGroup(group_ent));
        }
        // Mark as processed...
        commands.entity(group_ent).insert(RcGroupIsProcessed);
    }
}

/// Process [`RcSceneRoot`] when added.
pub fn process_scene_root_markers(
    scene_root_markers: Query<
        Entity,
        (
            With<RcSceneRoot>,
            Without<RcSceneRootIsProcessed>,
            Without<RcSceneRootReady>,
            Without<Cleanup>,
        ),
    >,
    children: Query<&Children, Without<Cleanup>>,
    mut commands: Commands,
) {
    for scene_root_ent in scene_root_markers.iter() {
        debug!("Processing RcSceneRoot: {:?}", scene_root_ent);
        let num_of_recursive_children = children.iter_descendants(scene_root_ent).count();
        if num_of_recursive_children == 0 {
            continue;
        }
        // Mark all children as belonging to this scene root...
        for child_ent in children.iter_descendants(scene_root_ent) {
            commands
                .entity(child_ent)
                .insert(BelongsToRcSceneRoot(scene_root_ent));
        }
        // Mark as processed...
        commands
            .entity(scene_root_ent)
            .insert(RcSceneRootIsProcessed);
    }
}

pub fn process_root_markers(
    mut commands: Commands,
    robot_roots: Query<
        Entity,
        (
            With<RcRobotRoot>,
            With<BelongsToRcSceneRoot>,
            Without<RcRobotRootIsProcessed>,
        ),
    >,
    children: Query<&Children, Without<Cleanup>>,
) {
    for root_ent in robot_roots.iter() {
        for child_ent in children.iter_descendants(root_ent) {
            commands
                .entity(child_ent)
                .insert(BelongsToRcRobotRoot(root_ent));
        }
        commands.entity(root_ent).insert((
            RcRobotGraph::default(),
            BelongsToRcRobotRoot(root_ent),
            // Mark as processed...
            RcRobotRootIsProcessed,
        ));
    }
}

pub fn process_collider_markers(
    mut commands: Commands,
    child_mesh_nodes: Query<(Entity, &Mesh3d, &ChildOf)>,
    collider_markers: Query<
        (
            Entity,
            &BelongsToRcRobotRoot,
            &RcCollider,
            Option<&RcColliderConstruction>,
        ),
        (
            With<RcCollider>,
            With<BelongsToRcSceneRoot>,
            Without<RcColliderIsProcessed>,
        ),
    >,
    robot_roots: Query<
        Entity,
        (
            With<RcRobotRoot>,
            With<BelongsToRcSceneRoot>,
            With<RcRobotRootIsProcessed>,
        ),
    >,
    meshes: Res<Assets<Mesh>>,
) {
    if collider_markers.is_empty() {
        return;
    }
    for (child_ent, mesh_3d, child_of) in child_mesh_nodes.iter() {
        let Ok((
            collider_ent,
            &BelongsToRcRobotRoot(root_ent),
            collider_marker,
            collider_construction,
        )) = collider_markers.get(child_of.parent())
        else {
            continue;
        };
        if !robot_roots.contains(root_ent) {
            warn!("Root not found for collider entity: {:?}", child_ent);
            continue;
        }
        let Some(mesh) = meshes.get(mesh_3d) else {
            continue;
        };
        let collider_construction = match collider_construction {
            Some(collider_construction) => match collider_construction {
                RcColliderConstruction::ConvexHull => ComputedColliderShape::ConvexHull,
                RcColliderConstruction::Trimesh => {
                    ComputedColliderShape::TriMesh(TriMeshFlags::all())
                }
                RcColliderConstruction::ConvexDecomposition => {
                    ComputedColliderShape::ConvexDecomposition(VHACDParameters::default())
                }
            },
            None => ComputedColliderShape::ConvexHull,
        };
        // create collider from mesh...
        let Some(collider) = Collider::from_bevy_mesh(mesh, &collider_construction) else {
            warn!(
                "Failed to create collider from mesh for entity: {:?}",
                child_ent
            );
            continue;
        };
        let RcCollider {
            density: collider_density,
            keep_mesh,
            ..
        } = collider_marker;
        // insert collider with density on node...
        commands
            .entity(collider_ent)
            .insert((collider, ColliderMassProperties::Density(*collider_density)));
        if !*keep_mesh {
            // Mark child entity for cleanup...
            commands.entity(child_ent).insert(Cleanup::Recursive);
        }
        // Mark as processed...
        commands.entity(collider_ent).insert(RcColliderIsProcessed);
    }
}

pub fn process_link_markers(
    mut commands: Commands,
    mut robot_roots: Query<
        &mut RcRobotGraph,
        (
            With<RcRobotRoot>,
            With<BelongsToRcSceneRoot>,
            With<RcRobotRootIsProcessed>,
        ),
    >,
    link_markers: Query<
        (
            Entity,
            &BelongsToRcRobotRoot,
            &RcLink,
            Option<&RcCcd>,
            Option<&RcDisableSleep>,
        ),
        (
            With<RcLink>,
            With<BelongsToRcSceneRoot>,
            Without<RcLinkIsProcessed>,
        ),
    >,
    children: Query<&Children, Without<Cleanup>>,
) {
    if link_markers.is_empty() {
        return;
    }
    for (link_ent, &BelongsToRcRobotRoot(root_ent), link_marker, rc_ccd, disable_sleep) in
        link_markers.iter()
    {
        let Ok(mut robot_graph) = robot_roots.get_mut(root_ent) else {
            warn!("Root not found for link entity: {:?}", link_ent);
            continue;
        };
        let RcLink { rigid_body, .. } = link_marker;
        commands
            .entity(link_ent)
            .insert((conversions::rigid_body_from_rc(rigid_body.clone()), Velocity::zero()));
        // Enable CCD if added...
        if rc_ccd.is_some() {
            commands.entity(link_ent).insert(Ccd::enabled());
        }
        if disable_sleep.is_some() {
            commands.entity(link_ent).insert(Sleeping::disabled());
        }
        // Mark children as belonging to this link...
        for child in children.iter_descendants(link_ent) {
            commands.entity(child).insert(BelongsToRcLink(link_ent));
        }
        // insert link in robot links map...
        robot_graph.add_link(RcRobotGraphLink {
            link_name: link_marker.name.clone(),
            link_entity: link_ent,
        });
        // Mark as processed...
        commands.entity(link_ent).insert(RcLinkIsProcessed);
    }
}

pub fn process_joint_markers(
    mut commands: Commands,
    mut robot_roots: Query<
        &mut RcRobotGraph,
        (
            With<RcRobotRoot>,
            With<BelongsToRcSceneRoot>,
            With<RcRobotRootIsProcessed>,
        ),
    >,
    joint_anchors: Query<
        (
            Entity,
            &ChildOf,
            &GlobalTransform,
            Has<RcJointLocalAnchor1>,
            Has<RcJointLocalAnchor2>,
        ),
        (
            Or<(With<RcJointLocalAnchor1>, With<RcJointLocalAnchor2>)>,
            With<BelongsToRcSceneRoot>,
        ),
    >,
    joint_markers: Query<
        (
            Entity,
            &BelongsToRcRobotRoot,
            &RcJoint,
            &RcJointKind,
            Option<&RcJointMotorControlConfig>,
            Option<&RcRevoluteJointMotor>,
            Option<&RcRevoluteJointLimits>,
            Option<&RcPrismaticJointMotor>,
            Option<&RcPrismaticJointLimits>,
            Option<&RcSphericalJointMotor>,
            Option<&RcSphericalJointLimits>,
        ),
        (
            With<RcJoint>,
            With<BelongsToRcSceneRoot>,
            Without<RcJointIsProcessed>,
            Without<Cleanup>,
        ),
    >,
    links: Query<
        (Entity, &BelongsToRcRobotRoot, &RcLink, &GlobalTransform),
        (With<RcLinkIsProcessed>, With<BelongsToRcSceneRoot>),
    >,
) {
    if joint_markers.is_empty() {
        return;
    }
    for (
        joint_ent,
        &BelongsToRcRobotRoot(root_ent),
        joint_marker,
        joint_kind,
        joint_motor_control_config,
        revolute_joint_motor,
        revolute_joint_limits,
        prismatic_joint_motor,
        prismatic_joint_limits,
        spherical_joint_motor,
        spherical_joint_limits,
    ) in joint_markers.iter()
    {
        let Ok(mut robot_graph) = robot_roots.get_mut(root_ent) else {
            warn!("Root not found for joint entity: {:?}", joint_ent);
            continue;
        };
        // Get data for joint creation...
        let (joint_parent_name, joint_child_name) = match joint_kind {
            RcJointKind::Fixed { parent, child } => (parent, child),
            RcJointKind::Revolute { parent, child, .. } => (parent, child),
            RcJointKind::Prismatic { parent, child, .. } => (parent, child),
            RcJointKind::Spherical { parent, child } => (parent, child),
        };
        let Some((parent_link, _, _, parent_link_glob_transform)) =
            links
                .iter()
                .find(|&(_, &BelongsToRcRobotRoot(cur_root_ent), ref link, _)| {
                    cur_root_ent == root_ent && link.name == *joint_parent_name
                })
        else {
            warn!("Parent link not found for joint: {:?}", joint_ent);
            continue;
        };
        let Some((child_link, _, _, child_link_glob_transform)) =
            links
                .iter()
                .find(|&(_, &BelongsToRcRobotRoot(cur_root_ent), ref link, _)| {
                    cur_root_ent == root_ent && link.name == *joint_child_name
                })
        else {
            warn!("Child link not found for joint: {:?}", joint_ent);
            continue;
        };
        let Some(local_anchor_1_glob_translation) = joint_anchors
            .iter()
            .find(|(_, child_of, _, is_anchor1, _)| *is_anchor1 && child_of.parent() == joint_ent)
            .map(|(_, _, global_transform, _, _)| global_transform.translation())
        else {
            warn!("Local anchor 1 not found for joint: {:?}", joint_ent);
            continue;
        };
        let Some(local_anchor_2_glob_translation) = joint_anchors
            .iter()
            .find(|(_, child_of, _, _, is_anchor2)| *is_anchor2 && child_of.parent() == joint_ent)
            .map(|(_, _, global_transform, _, _)| global_transform.translation())
        else {
            warn!("Local anchor 2 not found for joint: {:?}", joint_ent);
            continue;
        };
        debug!(
            "Global Anchor 1: {:?}, Global Anchor 2: {:?}",
            local_anchor_1_glob_translation, local_anchor_2_glob_translation
        );
        let local_anchor_1 = parent_link_glob_transform
            .to_matrix()
            .inverse()
            .transform_point3(local_anchor_1_glob_translation);
        let local_anchor_2 = child_link_glob_transform
            .to_matrix()
            .inverse()
            .transform_point3(local_anchor_2_glob_translation);
        debug!(
            "Local Anchor 1: {:?}, Local Anchor 2: {:?}",
            local_anchor_1, local_anchor_2
        );
        // create joint...
        let joint: TypedJoint = match joint_kind {
            RcJointKind::Fixed { .. } => {
                let joint = FixedJointBuilder::new()
                    .local_anchor1(local_anchor_1)
                    .local_anchor2(local_anchor_2);
                debug!("Fixed Joint created: {:?}", joint_ent);

                joint.into()
            }
            RcJointKind::Revolute { axis, .. } => {
                let mut joint = RevoluteJointBuilder::new(*axis)
                    .local_anchor1(local_anchor_1)
                    .local_anchor2(local_anchor_2);
                if let Some(RcRevoluteJointMotor {
                    model,
                    velocity,
                    position,
                    max_force,
                }) = revolute_joint_motor
                {
                    joint = joint.motor_model(conversions::motor_model_from_rd(model.clone()));
                    if let Some(velocity) = velocity {
                        joint = joint.motor_velocity(velocity.target, velocity.factor);
                    }
                    if let Some(position) = position {
                        joint = joint.motor_position(
                            position.target,
                            position.stiffness,
                            position.damping,
                        );
                    }
                    if let Some(max_force) = max_force {
                        joint = joint.motor_max_force(*max_force);
                    }
                }
                if let Some(RcRevoluteJointLimits(limits)) = revolute_joint_limits {
                    joint = joint.limits([limits.min, limits.max]);
                }
                debug!("Revolute Joint created: {:?}", joint_ent);

                joint.into()
            }
            RcJointKind::Prismatic { axis, .. } => {
                let mut joint = PrismaticJointBuilder::new(*axis)
                    .local_anchor1(local_anchor_1)
                    .local_anchor2(local_anchor_2);
                if let Some(RcPrismaticJointMotor {
                    model,
                    velocity,
                    position,
                    max_force,
                }) = prismatic_joint_motor
                {
                    joint = joint.motor_model(conversions::motor_model_from_rd(model.clone()));
                    if let Some(velocity) = velocity {
                        joint = joint.motor_velocity(velocity.target, velocity.factor);
                    }
                    if let Some(position) = position {
                        joint = joint.motor_position(
                            position.target,
                            position.stiffness,
                            position.damping,
                        );
                    }
                    if let Some(max_force) = max_force {
                        joint = joint.motor_max_force(*max_force);
                    }
                }
                if let Some(RcPrismaticJointLimits(limits)) = prismatic_joint_limits {
                    joint = joint.limits([limits.min, limits.max]);
                }
                debug!("Prismatic Joint created: {:?}", joint_ent);

                joint.into()
            }
            RcJointKind::Spherical { .. } => {
                let mut joint = SphericalJointBuilder::new()
                    .local_anchor1(local_anchor_1)
                    .local_anchor2(local_anchor_2);
                if let Some(RcSphericalJointMotor {
                    model_ang_x,
                    model_ang_y,
                    model_ang_z,
                    velocity_ang_x,
                    velocity_ang_y,
                    velocity_ang_z,
                    position_ang_x,
                    position_ang_y,
                    position_ang_z,
                    max_force_ang_x,
                    max_force_ang_y,
                    max_force_ang_z,
                }) = spherical_joint_motor
                {
                    joint = joint
                        .motor_model(
                            JointAxis::AngX,
                            conversions::motor_model_from_rd(model_ang_x.clone()),
                        )
                        .motor_model(
                            JointAxis::AngY,
                            conversions::motor_model_from_rd(model_ang_y.clone()),
                        )
                        .motor_model(
                            JointAxis::AngZ,
                            conversions::motor_model_from_rd(model_ang_z.clone()),
                        );
                    if let Some(velocity_ang_x) = velocity_ang_x {
                        joint = joint.motor_velocity(
                            JointAxis::AngX,
                            velocity_ang_x.target,
                            velocity_ang_x.factor,
                        );
                    }
                    if let Some(velocity_ang_y) = velocity_ang_y {
                        joint = joint.motor_velocity(
                            JointAxis::AngY,
                            velocity_ang_y.target,
                            velocity_ang_y.factor,
                        );
                    }
                    if let Some(velocity_ang_z) = velocity_ang_z {
                        joint = joint.motor_velocity(
                            JointAxis::AngZ,
                            velocity_ang_z.target,
                            velocity_ang_z.factor,
                        );
                    }
                    if let Some(position_ang_x) = position_ang_x {
                        joint = joint.motor_position(
                            JointAxis::AngX,
                            position_ang_x.target,
                            position_ang_x.stiffness,
                            position_ang_x.damping,
                        );
                    }
                    if let Some(position_ang_y) = position_ang_y {
                        joint = joint.motor_position(
                            JointAxis::AngY,
                            position_ang_y.target,
                            position_ang_y.stiffness,
                            position_ang_y.damping,
                        );
                    }
                    if let Some(position_ang_z) = position_ang_z {
                        joint = joint.motor_position(
                            JointAxis::AngZ,
                            position_ang_z.target,
                            position_ang_z.stiffness,
                            position_ang_z.damping,
                        );
                    }
                    if let Some(max_force_ang_x) = max_force_ang_x {
                        joint = joint.motor_max_force(JointAxis::AngX, *max_force_ang_x);
                    }
                    if let Some(max_force_ang_y) = max_force_ang_y {
                        joint = joint.motor_max_force(JointAxis::AngY, *max_force_ang_y);
                    }
                    if let Some(max_force_ang_z) = max_force_ang_z {
                        joint = joint.motor_max_force(JointAxis::AngZ, *max_force_ang_z);
                    }
                }
                if let Some(RcSphericalJointLimits {
                    ang_x,
                    ang_y,
                    ang_z,
                }) = spherical_joint_limits
                {
                    if let Some(ang_x) = ang_x {
                        joint = joint.limits(JointAxis::AngX, [ang_x.min, ang_x.max]);
                    }
                    if let Some(ang_y) = ang_y {
                        joint = joint.limits(JointAxis::AngY, [ang_y.min, ang_y.max]);
                    }
                    if let Some(ang_z) = ang_z {
                        joint = joint.limits(JointAxis::AngZ, [ang_z.min, ang_z.max]);
                    }
                }
                debug!("Spherical Joint created: {:?}", joint_ent);

                joint.into()
            }
        };
        // insert joint on child entity...
        let mut child_ent_commands = commands.entity(child_link);
        match joint_marker.category {
            RcJointCategory::Impulse => {
                child_ent_commands.insert(ImpulseJoint::new(parent_link, joint));
            }
            RcJointCategory::Multibody => {
                child_ent_commands.insert(MultibodyJoint::new(parent_link, joint));
            }
        }
        // insert joint motor control config if added...
        if let Some(joint_motor_control_config) = joint_motor_control_config {
            child_ent_commands.insert(joint_motor_control_config.clone());
        }

        // insert joint in robot joints map...
        robot_graph.add_joint(RcRobotGraphJoint {
            joint_name: joint_marker.name.clone(),
            joint_entity: child_link,
            from_link: joint_parent_name.clone(),
            to_link: joint_child_name.clone(),
        });

        // Mark temp joint markers for cleanup...
        commands.entity(joint_ent).insert(Cleanup::Recursive);
    }
}

pub fn check_processing_complete(
    mut commands: Commands,
    scene_root_markers: Query<
        (Entity, Option<&RcSceneRootIsProcessed>),
        (With<RcSceneRoot>, Without<RcSceneRootReady>),
    >,
    root_markers: Query<
        (
            Entity,
            &BelongsToRcSceneRoot,
            Option<&RcRobotRootIsProcessed>,
        ),
        With<RcRobotRoot>,
    >,
    collider_markers: Query<
        (
            Entity,
            &BelongsToRcSceneRoot,
            Option<&RcColliderIsProcessed>,
        ),
        With<RcCollider>,
    >,
    link_markers: Query<(Entity, &BelongsToRcSceneRoot, Option<&RcLinkIsProcessed>), With<RcLink>>,
    joint_markers: Query<
        (Entity, &BelongsToRcSceneRoot, Option<&RcJointIsProcessed>),
        With<RcJoint>,
    >,
    // ik_targets: Query<Entity, (With<RcIkTarget>, Without<RcIsProcessed>)>,
    // ik_sources: Query<Entity, (With<RcIkTargetSource>, Without<RcIsProcessed>)>,
) {
    for (scene_root_ent, scene_root_is_processed) in scene_root_markers.iter() {
        // Scene Root...
        if scene_root_is_processed.is_none() {
            continue;
        }
        // Root markers...
        let all_root_markers_processed = root_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_root_markers_processed {
            continue;
        }
        // Collider markers...
        let all_collider_markers_processed = collider_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_collider_markers_processed {
            continue;
        }
        // Link markers...
        let all_link_markers_processed = link_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_link_markers_processed {
            continue;
        }
        // joint markers...
        let all_joint_markers_processed = joint_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_joint_markers_processed {
            continue;
        }
        debug!("scene_root_ent: {} is ready!!", scene_root_ent);
        // Mark the scene ready...
        commands.entity(scene_root_ent).insert(RcSceneRootReady);
    }
}

/// Detaches root markers and attaches children to the game world directly to flatten the hierarchy.
///
/// Should happen only for newly added [`RcSceneRootReady`] scenes.
pub fn detach_root_markers_and_flatten_hierarchy(
    mut commands: Commands,
    scene_root_markers: Query<
        Entity,
        (
            With<RcSceneRoot>,
            With<RcSceneRootIsProcessed>,
            With<RcSceneRootReady>,
            Without<Cleanup>,
            Without<RcSceneRootWRobotRootsDetached>,
        ),
    >,
    root_markers: Query<
        (Entity, &BelongsToRcSceneRoot),
        (With<RcRobotRoot>, With<RcRobotRootIsProcessed>),
    >,
    children: Query<(Entity, &ChildOf), Without<Cleanup>>,
) {
    for scene_root in scene_root_markers.iter() {
        let root_markers_belonging_to_scene_root = root_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root))| cur_scene_root == scene_root);
        if root_markers_belonging_to_scene_root.count() == 0 {
            continue;
        }
        for (root_ent, &BelongsToRcSceneRoot(cur_scene_root)) in root_markers.iter() {
            if cur_scene_root != scene_root {
                continue;
            }
            for (child_ent, child_of) in children.iter() {
                if child_of.parent() != root_ent {
                    continue;
                }
                commands.entity(child_ent).remove_parent_in_place();
            }
        }
        // Mark as detached...
        commands
            .entity(scene_root)
            .insert(RcSceneRootWRobotRootsDetached);
    }
}
