use avian3d::prelude::*;
use bevy::prelude::*;

use crate::conversions::{
    self, fixed_joint_from_rd, prismatic_joint_from_rd, revolute_joint_from_rd,
    spherical_joint_from_rd,
};
use robocomp::cleanup_manager::Cleanup;

use robocomp::rc::{
    BelongsToRcGroup, BelongsToRcLink, BelongsToRcRobotRoot, BelongsToRcSceneRoot, RcCcd,
    RcCollider, RcColliderConstruction, RcColliderIsProcessed, RcDisableSleep, RcGroup,
    RcGroupIsProcessed, RcJoint, RcJointIsProcessed, RcJointKind, RcJointLocalAnchor1,
    RcJointLocalAnchor2, RcJointMotorControlConfig, RcLink, RcLinkIsProcessed,
    RcPrismaticJointLimits, RcPrismaticJointMotor, RcRevoluteJointLimits, RcRevoluteJointMotor,
    RcRobotGraph, RcRobotGraphJoint, RcRobotGraphLink, RcRobotRoot, RcRobotRootIsProcessed,
    RcSceneRoot, RcSceneRootIsProcessed, RcSceneRootReady, RcSceneRootWRobotRootsDetached,
    RcSphericalJointLimits, RcSphericalJointMotor,
};
use robocomp::rd::{
    RdFixedJointData, RdMotor, RdPrismaticJointData, RdRevoluteJointData,
    RdSphericalJointData,
};

/// Processes marked entities with compositor components.
pub struct PreProcessorPlugin;

impl Plugin for PreProcessorPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, PreProcessorPluginSystemSet)
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
        for child_ent in children.iter_descendants(group_ent) {
            commands
                .entity(child_ent)
                .insert(BelongsToRcGroup(group_ent));
        }
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
        for child_ent in children.iter_descendants(scene_root_ent) {
            commands
                .entity(child_ent)
                .insert(BelongsToRcSceneRoot(scene_root_ent));
        }
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
        let collider = match collider_construction {
            Some(RcColliderConstruction::ConvexHull) | None => {
                Collider::convex_hull_from_mesh(mesh)
            }
            Some(RcColliderConstruction::Trimesh) => Collider::trimesh_from_mesh(mesh),
            Some(RcColliderConstruction::ConvexDecomposition) => {
                Collider::convex_decomposition_from_mesh(mesh)
            }
        };
        let Some(collider) = collider else {
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
        commands.entity(collider_ent).insert((
            collider,
            ColliderDensity(*collider_density),
        ));
        if !*keep_mesh {
            commands.entity(child_ent).insert(Cleanup::Recursive);
        }
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
    for (link_ent, &BelongsToRcRobotRoot(root_ent), link_marker, _rc_ccd, disable_sleep) in
        link_markers.iter()
    {
        let Ok(mut robot_graph) = robot_roots.get_mut(root_ent) else {
            warn!("Root not found for link entity: {:?}", link_ent);
            continue;
        };
        let RcLink { rigid_body, .. } = link_marker;
        commands
            .entity(link_ent)
            .insert(conversions::rigid_body_from_rc(rigid_body.clone()));
        if disable_sleep.is_some() {
            commands.entity(link_ent).insert(SleepingDisabled);
        }
        for child in children.iter_descendants(link_ent) {
            commands.entity(child).insert(BelongsToRcLink(link_ent));
        }
        robot_graph.add_link(RcRobotGraphLink {
            link_name: link_marker.name.clone(),
            link_entity: link_ent,
        });
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
        _spherical_joint_motor,
        spherical_joint_limits,
    ) in joint_markers.iter()
    {
        let Ok(mut robot_graph) = robot_roots.get_mut(root_ent) else {
            warn!("Root not found for joint entity: {:?}", joint_ent);
            continue;
        };
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
        let local_anchor_1 = parent_link_glob_transform
            .to_matrix()
            .inverse()
            .transform_point3(local_anchor_1_glob_translation);
        let local_anchor_2 = child_link_glob_transform
            .to_matrix()
            .inverse()
            .transform_point3(local_anchor_2_glob_translation);

        let default_motor = RdMotor {
            model: robocomp::rd::RdMotorModel::AccelerationBased,
            velocity: None,
            position: None,
            max_force: None,
        };

        let joint_entity = match joint_kind {
            RcJointKind::Fixed { .. } => commands
                .spawn(fixed_joint_from_rd(
                    parent_link,
                    child_link,
                    RdFixedJointData {
                        local_anchor1: local_anchor_1,
                        local_anchor2: local_anchor_2,
                    },
                ))
                .id(),
            RcJointKind::Revolute { axis, .. } => {
                let motor = revolute_joint_motor
                    .map(|RcRevoluteJointMotor {
                        model,
                        velocity,
                        position,
                        max_force,
                    }| RdMotor {
                        model: model.clone(),
                        velocity: velocity.clone(),
                        position: position.clone(),
                        max_force: *max_force,
                    })
                    .unwrap_or_else(|| default_motor.clone());
                let limits = revolute_joint_limits
                    .map(|RcRevoluteJointLimits(limits)| limits.clone());
                commands
                    .spawn(revolute_joint_from_rd(
                        parent_link,
                        child_link,
                        RdRevoluteJointData {
                            axis: *axis,
                            local_anchor1: local_anchor_1,
                            local_anchor2: local_anchor_2,
                            motor,
                            limits,
                        },
                    ))
                    .id()
            }
            RcJointKind::Prismatic { axis, .. } => {
                let motor = prismatic_joint_motor
                    .map(|RcPrismaticJointMotor {
                        model,
                        velocity,
                        position,
                        max_force,
                    }| RdMotor {
                        model: model.clone(),
                        velocity: velocity.clone(),
                        position: position.clone(),
                        max_force: *max_force,
                    })
                    .unwrap_or_else(|| default_motor.clone());
                let limits = prismatic_joint_limits
                    .map(|RcPrismaticJointLimits(limits)| limits.clone());
                commands
                    .spawn(prismatic_joint_from_rd(
                        parent_link,
                        child_link,
                        RdPrismaticJointData {
                            axis: *axis,
                            local_anchor1: local_anchor_1,
                            local_anchor2: local_anchor_2,
                            motor,
                            limits,
                        },
                    ))
                    .id()
            }
            RcJointKind::Spherical { .. } => {
                let limit_ang_x = spherical_joint_limits.and_then(|l| l.ang_x.clone());
                let limit_ang_y = spherical_joint_limits.and_then(|l| l.ang_y.clone());
                let limit_ang_z = spherical_joint_limits.and_then(|l| l.ang_z.clone());
                commands
                    .spawn(spherical_joint_from_rd(
                        parent_link,
                        child_link,
                        RdSphericalJointData {
                            local_anchor1: local_anchor_1,
                            local_anchor2: local_anchor_2,
                            motor_x: None,
                            motor_y: None,
                            motor_z: None,
                            limit_ang_x,
                            limit_ang_y,
                            limit_ang_z,
                        },
                    ))
                    .id()
            }
        };

        if let Some(joint_motor_control_config) = joint_motor_control_config {
            commands
                .entity(joint_entity)
                .insert(joint_motor_control_config.clone());
        }

        robot_graph.add_joint(RcRobotGraphJoint {
            joint_name: joint_marker.name.clone(),
            joint_entity,
            from_link: joint_parent_name.clone(),
            to_link: joint_child_name.clone(),
        });

        commands.entity(joint_ent).insert((
            RcJointIsProcessed,
            Cleanup::Recursive,
        ));
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
) {
    for (scene_root_ent, scene_root_is_processed) in scene_root_markers.iter() {
        if scene_root_is_processed.is_none() {
            continue;
        }
        let all_root_markers_processed = root_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_root_markers_processed {
            continue;
        }
        let all_collider_markers_processed = collider_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_collider_markers_processed {
            continue;
        }
        let all_link_markers_processed = link_markers
            .iter()
            .filter(|&(_, &BelongsToRcSceneRoot(cur_scene_root), _)| {
                cur_scene_root == scene_root_ent
            })
            .all(|(_, _, is_processed)| is_processed.is_some());
        if !all_link_markers_processed {
            continue;
        }
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
        commands.entity(scene_root_ent).insert(RcSceneRootReady);
    }
}

/// Detaches root markers and attaches children to the game world directly to flatten the hierarchy.
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
        commands
            .entity(scene_root)
            .insert(RcSceneRootWRobotRootsDetached);
    }
}
