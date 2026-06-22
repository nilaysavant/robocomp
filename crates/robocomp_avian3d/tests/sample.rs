mod avian_backend;

use bevy::prelude::*;
use avian_backend::AvianBackend;
use robocomp_test_utils::{run_sample_scene_serialize_test, run_sample_scene_test};

#[test]
fn sample_test() -> Result<()> {
    run_sample_scene_test::<AvianBackend>()
}

#[test]
fn scene_serialize_test() -> Result<()> {
    run_sample_scene_serialize_test::<AvianBackend>()
}
