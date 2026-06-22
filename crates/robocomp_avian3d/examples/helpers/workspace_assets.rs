//! Workspace asset path helper for Skein examples.

use std::path::Path;

/// Path to workspace-level `assets/` from the `robocomp_avian3d` crate.
pub fn workspace_assets_dir() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .to_string_lossy()
        .into_owned()
}
