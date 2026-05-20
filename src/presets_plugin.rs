use bevy::prelude::*;
use bevy_skein::SkeinAppExt;

use crate::rc::RcCollider;

/// # Presets Plugin
///
/// Plugin for Skein presets for use in Blender.
pub struct PresetsPlugin;

impl Plugin for PresetsPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .insert_skein_preset(RcColliderKeepMesh::name(), RcColliderKeepMesh::value())
            .add_systems(Startup, || info!("PresetsPlugin started..."));
    }
}

/// Trait for custom presets.
pub trait CustomPreset {
    /// Component type for the preset.
    type PresetType: Component + Reflect + Default;

    /// Name of the preset.
    fn name() -> &'static str;

    /// Value of the preset.
    fn value() -> Self::PresetType;
}

pub struct RcColliderKeepMesh;

impl CustomPreset for RcColliderKeepMesh {
    type PresetType = RcCollider;

    fn name() -> &'static str {
        "RcColliderKeepMesh"
    }

    fn value() -> Self::PresetType {
        RcCollider {
            keep_mesh: true,
            ..Default::default()
        }
    }
}
