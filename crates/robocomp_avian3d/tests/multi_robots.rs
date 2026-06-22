mod avian_backend;

use bevy::prelude::*;
use avian_backend::AvianBackend;
use robocomp_test_utils::run_multi_robots_test;

#[test]
fn multi_robots() -> Result<()> {
    run_multi_robots_test::<AvianBackend>()
}
