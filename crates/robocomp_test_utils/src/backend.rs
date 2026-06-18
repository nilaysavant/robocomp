use bevy::prelude::*;

/// Physics-backend hooks used by shared integration test runners.
pub trait PhysicsTestBackend {
    fn add_physics_plugins(app: &mut App);
    fn validate_colliders(world: &mut World) -> Result<()>;
    fn validate_links(world: &mut World) -> Result<()>;
    fn collider_proxy_type_path() -> &'static str;
}
