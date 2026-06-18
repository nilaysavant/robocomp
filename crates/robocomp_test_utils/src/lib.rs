#![allow(clippy::type_complexity)]

mod backend;
mod harness;
mod runners;
pub mod scenarios;

pub use backend::PhysicsTestBackend;
pub use harness::{
    BevyTestingDefaultsPlugin, RobocompTestHarnessPlugin, serialize_scene_world,
    wait_for_scene_ready,
};
pub use runners::{run_multi_robots_test, run_sample_scene_serialize_test, run_sample_scene_test};
