use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{platform::collections::HashMap, prelude::*};
use petgraph::prelude::*;

use super::rd::{
    RdJointLimits, RdMotorModel, RdMotorPosition, RdMotorVelocity, RdName, RdScaler, RdVector,
};

/// Root Marker for the Scene containing the robot(s).
///
/// Add manually to the [`SceneRoot`] only while spawning the scene.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcSceneRoot;

/// Indicates which [`RcSceneRoot`] this part belongs to.
///
/// Not to be added in Blender.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct BelongsToRcSceneRoot(pub Entity);

/// Marker indicating that the scene root entity has been processed by compositor.
///
/// Not exposed to Blender.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcSceneRootIsProcessed;

/// Marker for when an [`RcSceneRoot`] is ready!
///
/// - This means all children/parts of the scene (incl itself) have
///   been processed by the compositor.
///
/// Not exposed to [`bevy_skein`].
#[derive(Debug, Reflect, Component)]
pub struct RcSceneRootReady;

/// Marker for when an [`RcSceneRoot`] has all child [`RcRobotRoot`]s detached.
///
/// Not exposed to [`bevy_skein`].
#[derive(Debug, Reflect, Component)]
pub struct RcSceneRootWRobotRootsDetached;

/// Root Marker for the Group containing the robot(s).
///
/// Add manually to the root/parent of a set of robots (children) to be grouped together.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcGroup;

/// Indicates which [`RcGroup`] this part belongs to.
///
/// Not to be added in Blender.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct BelongsToRcGroup(pub Entity);

/// Marker indicating that the group entity has been processed by compositor.
///
/// Not exposed to Blender.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcGroupIsProcessed;

/// Root Marker for the Robot.
///
/// Add to the root node/empty that groups all the robot
/// parts under a this single parent.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcRobotRoot;

/// Marker indicating that the robot root entity has been processed by compositor.
///
/// Not exposed to Blender.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcRobotRootIsProcessed;

/// Robot Links/Joints Graph.
///
/// Auto added to [`RcRobotRoot`] entity.
///
/// Not exposed to Blender.
#[derive(Debug, Clone, Reflect, Component, Default)]
pub struct RcRobotGraph {
    /// Petgraph graph of the robot.
    ///
    /// Where:
    /// - `Nodes` are the [`RcLink`] [`Entity`]s.
    /// - `Edges` are the [`RcJoint`]s' linking the [`RcLink`] [`Entity`]s.
    #[reflect(ignore)]
    graph: StableGraph<RcRobotGraphLink, RcRobotGraphJoint, Directed>,
    /// Link nodes stored by their name.
    #[reflect(ignore)]
    link_nodes_by_name: HashMap<RdName, NodeIndex>,
    /// Joint edges stored by their name.
    #[reflect(ignore)]
    joints_edges_by_name: HashMap<RdName, EdgeIndex>,
}

/// Robot Graph Link Node.
#[derive(Debug, Clone, Reflect, Component)]
pub struct RcRobotGraphLink {
    /// Name of the link.
    pub link_name: RdName,
    /// [`Entity`] of the link.
    pub link_entity: Entity,
}

/// Robot Graph Joint Edge.
#[derive(Debug, Clone, Reflect, Component)]
pub struct RcRobotGraphJoint {
    /// Name of the joint.
    pub joint_name: RdName,
    /// [`Entity`] on which the physics joint is set.
    pub joint_entity: Entity,
    /// From link name.
    pub from_link: RdName,
    /// To link name.
    pub to_link: RdName,
}

impl RcRobotGraph {
    /// Add a link to the graph with the given name.
    ///
    /// Returns the node index of the added link.
    pub fn add_link(&mut self, link: RcRobotGraphLink) -> Option<NodeIndex> {
        let node = self.graph.add_node(link.clone());
        self.link_nodes_by_name.insert(link.link_name, node);

        Some(node)
    }

    /// Get all the links in the graph.
    ///
    /// Returns an iterator of the links.
    pub fn links(&self) -> impl Iterator<Item = &RcRobotGraphLink> {
        self.graph.node_weights()
    }

    /// Get a link from the graph by its name.
    pub fn get_link(&self, name: &RdName) -> Option<&RcRobotGraphLink> {
        let node = self.link_nodes_by_name.get(name)?;
        let link = self.graph.node_weight(*node);

        link
    }

    /// Remove a link from the graph by its name.
    ///
    /// Returns the removed link and the removed joints that were connected to it. Else None if failed to remove.
    pub fn remove_link(
        &mut self,
        name: &RdName,
    ) -> Option<(RcRobotGraphLink, Vec<RcRobotGraphJoint>)> {
        let node = self.link_nodes_by_name.remove(name)?;
        let edges_to_remove = self
            .graph
            .edges(node)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();
        let mut removed_joints = vec![];
        for edge in edges_to_remove {
            let Some(joint) = self.graph.remove_edge(edge) else {
                continue;
            };
            // Remove the joint from the joints_edges_by_name.
            self.joints_edges_by_name.remove(&joint.joint_name);
            // Store the removed joint.
            removed_joints.push(joint);
        }
        // Remove the link from the graph.
        let link = self.graph.remove_node(node)?;

        Some((link, removed_joints))
    }

    /// Add a joint to the graph with the given name.
    pub fn add_joint(&mut self, joint: RcRobotGraphJoint) -> Option<EdgeIndex> {
        let RcRobotGraphJoint {
            joint_name,
            from_link,
            to_link,
            ..
        } = &joint;
        let from_node = self.link_nodes_by_name.get(from_link)?;
        let to_node = self.link_nodes_by_name.get(to_link)?;
        let edge = self.graph.add_edge(*from_node, *to_node, joint.clone());
        self.joints_edges_by_name.insert(joint_name.clone(), edge);

        Some(edge)
    }

    /// Get all the joints in the graph.
    ///
    /// Returns an iterator of the joints.
    pub fn joints(&self) -> impl Iterator<Item = &RcRobotGraphJoint> {
        self.graph.edge_weights()
    }

    /// Get a joint from the graph by its name.
    ///
    /// Returns the joint. Else None if not found.
    pub fn get_joint(&self, name: &RdName) -> Option<&RcRobotGraphJoint> {
        let edge = self.joints_edges_by_name.get(name)?;
        let joint = self.graph.edge_weight(*edge);

        joint
    }

    /// Get all the joints on a link by its name.
    ///
    /// Returns an iterator of the joints on the link. Else None if not found.
    pub fn get_joints_on_link(
        &self,
        link: &RdName,
    ) -> Option<impl IntoIterator<Item = &RcRobotGraphJoint>> {
        let Some(node) = self.link_nodes_by_name.get(link) else {
            warn!("Link not found: {:?}", link);
            return None;
        };
        let edges = self.graph.edges(*node).map(|edge| edge.id());
        let joints = edges.filter_map(|edge| self.graph.edge_weight(edge));

        Some(joints)
    }

    /// Get the joint connecting the two links by their names.
    ///
    /// Returns the joint connecting the two links. Else None if not found.
    pub fn get_joint_connecting(
        &self,
        from_link: &RdName,
        to_link: &RdName,
    ) -> Option<&RcRobotGraphJoint> {
        let from_node = self.link_nodes_by_name.get(from_link)?;
        let to_node = self.link_nodes_by_name.get(to_link)?;
        let edge = self.graph.find_edge(*from_node, *to_node)?;
        let joint = self.graph.edge_weight(edge)?;

        Some(joint)
    }

    /// Remove a joint from the graph by its name.
    ///
    /// Returns the removed joint. Else None if failed to remove.
    pub fn remove_joint(&mut self, name: &RdName) -> Option<RcRobotGraphJoint> {
        let edge = self.joints_edges_by_name.remove(name)?;
        let joint = self.graph.remove_edge(edge)?;

        Some(joint)
    }
}

/// Indicates to which [`RcRobotRoot`] this part belongs to.
///
/// Not exposed to Blenvy.
#[derive(Debug, Clone, Reflect, Component)]
pub struct BelongsToRcRobotRoot(pub Entity);

/// Link Marker for the Robot.
///
/// Add to the Link nodes of the robot.
#[derive(Debug, Clone, Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct RcLink {
    /// Name of the link.
    pub name: RdName,
    pub rigid_body: RcRigidBody,
}

/// Indicates to which [`RcLink`] this part belongs to.
///
/// Not exposed to Blenvy.
#[derive(Debug, Clone, Reflect, Component)]
pub struct BelongsToRcLink(pub Entity);

/// Marks the root [`RcLink`] of the robot.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
#[require(RcLink = RcLink::default())]
pub struct RcLinkRoot;

/// Marker indicating that the link entity has been processed by compositor.
///
/// Not exposed to Blenvy.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcLinkIsProcessed;

/// Rigid Body Type.
#[derive(Debug, Clone, Reflect, Default)]
pub enum RcRigidBody {
    #[default]
    Fixed,
    Dynamic,
}

/// Add continuous collision detection to the rigid body.
///
/// To be added on same node as [`RcLink`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcCcd;

/// Disable sleeping for the rigid body.
///
/// To be added on same node as [`RcLink`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcDisableSleep;

/// Collider Marker for the Robot.
///
/// Add to the collider node (direct parent of the collider mesh).
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component, Default)]
#[require(RcColliderConstruction = RcColliderConstruction::default())]
pub struct RcCollider {
    /// Collider density.
    pub density: RdScaler,
    /// Keep the collider mesh after the collider is created.
    pub keep_mesh: bool,
}

impl Default for RcCollider {
    fn default() -> Self {
        Self {
            density: 1.0,
            keep_mesh: true,
        }
    }
}

/// Collider Marker for the Robot.
///
/// Add to the collider node (direct parent of the collider mesh).
#[derive(Debug, Clone, Reflect, Default, Component)]
#[reflect(Component)]
pub enum RcColliderConstruction {
    #[default]
    Trimesh,
    ConvexHull,
    ConvexDecomposition,
}

/// Marker indicating that the collider entity has been processed by compositor.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcColliderIsProcessed;

/// Joint Marker.
///
/// Add to an empty node that will serve as a joint.
#[derive(Debug, Clone, Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct RcJoint {
    pub name: RdName,
    pub category: RcJointCategory,
}

/// Marker indicating that the joint entity has been processed by compositor.
///
/// Not exposed to Blenvy.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcJointIsProcessed;

/// Joint Category (Impulse/Multibody) for the [`RcJoint`].
#[derive(Debug, Clone, Reflect, Default)]
pub enum RcJointCategory {
    /// For Normal joints.
    #[default]
    Impulse,
    // For Inverse Kinematics joints.
    Multibody,
}

/// Joint Kind for the RcJoint.
///
/// Add to same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub enum RcJointKind {
    Fixed {
        parent: RdName,
        child: RdName,
    },
    Revolute {
        parent: RdName,
        child: RdName,
        /// Axis of the joint.
        axis: RdVector,
    },
    Prismatic {
        parent: RdName,
        child: RdName,
        /// Axis of the joint.
        axis: RdVector,
    },
    Spherical {
        parent: RdName,
        child: RdName,
    },
}

/// Config for Control of the Joint Motor.
///
/// Optionally add to the same node as [`RcJoint`].
/// Also requires [`RcRevoluteJointMotor`] or equivalent to be added to the same node.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub enum RcJointMotorControlConfig {
    /// Config for Position Control of the Joint Motor.
    Position {
        /// Boost Angular velocity of the joint motor(s).
        ///
        /// This is added every physics update to the motor's position during control action.
        boost_ang_vel: f32,
        /// Normal Angular velocity of the joint motor(s).
        ///
        /// This is added every physics update to the motor's position during control action.
        norm_ang_vel: f32,
        /// Stiffness set on the joint motor(s).
        stiffness: f32,
        /// Damping set on the joint motor(s).
        damping: f32,
    },
    /// Config for Velocity Control of the Joint Motor.
    Velocity {
        normal_vel: f32,
        normal_factor: f32,
        boost_vel: f32,
        boost_factor: f32,
    },
}

impl Default for RcJointMotorControlConfig {
    fn default() -> Self {
        Self::Position {
            boost_ang_vel: PI,       // 180 degrees per second.
            norm_ang_vel: FRAC_PI_2, // 90 degrees per second.
            stiffness: 5000.0,
            damping: 650.0,
        }
    }
}

/// Configuring Revolute Joint Motor.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcRevoluteJointMotor {
    pub model: RdMotorModel,
    pub velocity: Option<RdMotorVelocity>,
    pub position: Option<RdMotorPosition>,
    pub max_force: Option<RdScaler>,
}

/// Configuring Revolute Joint Limits.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcRevoluteJointLimits(pub RdJointLimits);

/// Configuring Prismatic Joint Motor.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcPrismaticJointMotor {
    pub model: RdMotorModel,
    pub velocity: Option<RdMotorVelocity>,
    pub position: Option<RdMotorPosition>,
    pub max_force: Option<RdScaler>,
}

/// Configuring Prismatic Joint Limits.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcPrismaticJointLimits(pub RdJointLimits);

/// Configuring Spherical Joint Motor.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcSphericalJointMotor {
    // Model x, y, z
    pub model_ang_x: RdMotorModel,
    pub model_ang_y: RdMotorModel,
    pub model_ang_z: RdMotorModel,
    // Velocity x, y, z
    pub velocity_ang_x: Option<RdMotorVelocity>,
    pub velocity_ang_y: Option<RdMotorVelocity>,
    pub velocity_ang_z: Option<RdMotorVelocity>,
    // Position x, y, z
    pub position_ang_x: Option<RdMotorPosition>,
    pub position_ang_y: Option<RdMotorPosition>,
    pub position_ang_z: Option<RdMotorPosition>,
    // Max Force x, y, z
    pub max_force_ang_x: Option<RdScaler>,
    pub max_force_ang_y: Option<RdScaler>,
    pub max_force_ang_z: Option<RdScaler>,
}

/// Configuring Spherical Joint Limits.
///
/// Add to the same node as [`RcJoint`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcSphericalJointLimits {
    pub ang_x: Option<RdJointLimits>,
    pub ang_y: Option<RdJointLimits>,
    pub ang_z: Option<RdJointLimits>,
}

/// Joint Local Anchor 1.
///
/// Add to a child empty/node of [`RcJoint`].
///
/// The transform of this node will be used as the anchor point.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcJointLocalAnchor1;

/// Joint Local Anchor 1.
///
/// Add to a child empty/node of [`RcJoint`].
///
/// The transform of this node will be used as the anchor point.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcJointLocalAnchor2;

/// Inverse Kinematics Target marker.
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcIkTarget {
    pub name: RdName,
}

/// Inverse Kinematics Source marker.
///
/// To be added on a [`RcLink`] which will track the target marked by [`RcIkTarget`]. Given its [`RdName`].
#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct RcIkTargetSource {
    pub target: RdName,
}

/// ALl links that should be affected by the target including this link.
/// If no links are provided, all links will be affected.
///
/// To be added on a [`RcIkTargetSource`].
#[derive(Debug, Clone, Reflect, Deref, DerefMut, Component)]
#[reflect(Component)]
pub struct RcIkAffectedLinks(pub Vec<RdName>);
