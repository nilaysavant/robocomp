use bevy::{ecs::error::warn, prelude::*};

pub struct CleanupManagerPlugin;

impl Plugin for CleanupManagerPlugin {
    fn build(&self, app: &mut App) {
        app // App
            .register_type::<Cleanup>()
            .add_systems(First, cleanup)
            .add_systems(Startup, || info!("CleanupManagerPlugin started..."));
    }
}

/// Component used to mark entities for cleanup.
#[derive(Debug, Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub enum Cleanup {
    /// Normal cleanup. Cleans'up/deletes only itself.
    Normal,
    /// Cleanup itself and all descendants recursively.
    #[default]
    Recursive,
    /// Cleanup only descendants.
    Descendants,
}

/// Cleanup routine system.
pub fn cleanup(mut commands: Commands, query: Query<(Entity, &Cleanup)>) {
    for (entity, cleanup) in query.iter() {
        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };
        match cleanup {
            Cleanup::Normal => {
                entity_commands.queue_handled(
                    |mut cmds: EntityWorldMut| -> Result {
                        // First remove the children relation from the entity, then despawn it...
                        cmds.remove::<Children>();
                        cmds.despawn();
                        Ok(())
                    },
                    warn,
                );
            }
            Cleanup::Recursive => {
                entity_commands.queue_handled(
                    |cmds: EntityWorldMut| -> Result {
                        cmds.despawn();
                        Ok(())
                    },
                    warn,
                );
            }
            Cleanup::Descendants => {
                entity_commands.queue_handled(
                    |mut cmds: EntityWorldMut| -> Result {
                        cmds.despawn_related::<Children>();
                        Ok(())
                    },
                    warn,
                );
            }
        }
    }
}
