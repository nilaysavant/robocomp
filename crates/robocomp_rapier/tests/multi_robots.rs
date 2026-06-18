mod rapier_backend;

use bevy::prelude::*;
use rapier_backend::RapierBackend;
use robocomp_test_utils::run_multi_robots_test;

#[test]
fn multi_robots() -> Result<()> {
    run_multi_robots_test::<RapierBackend>()
}
