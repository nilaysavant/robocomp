mod rapier_backend;

use bevy::prelude::*;
use rapier_backend::RapierBackend;
use robocomp_test_utils::{run_sample_scene_serialize_test, run_sample_scene_test};

#[test]
fn sample_test() -> Result<()> {
    run_sample_scene_test::<RapierBackend>()
}

#[test]
fn scene_serialize_test() -> Result<()> {
    run_sample_scene_serialize_test::<RapierBackend>()
}
