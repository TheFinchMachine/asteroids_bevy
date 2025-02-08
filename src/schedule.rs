use bevy::prelude::*;

use crate::GameState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InGameSet {
    GameInput,
    MenuInput,
    EntityUpdates,
    CollisionDetection,
    RenderSetup,
    DespawnEntities,
}

pub struct SchudulePlugin;

impl Plugin for SchudulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                InGameSet::DespawnEntities,
                // apply_deferred(Flush)
                InGameSet::GameInput,
                InGameSet::EntityUpdates,
                InGameSet::RenderSetup,
                InGameSet::CollisionDetection,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
        app.add_systems(
            Update,
            apply_deferred
                .after(InGameSet::DespawnEntities)
                .before(InGameSet::GameInput),
        );
        app.configure_sets(Update, (InGameSet::MenuInput));
    }
}
