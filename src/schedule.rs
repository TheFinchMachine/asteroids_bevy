use bevy::prelude::*;

use crate::GameState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InGameSet {
    GameInput,
    MenuInput,
    LoadEntities,
    DespawnEntities,
    UpdateEntities,
    CollisionDetection,
    CollisionReaction,
    RenderSetup,
}

pub struct SchudulePlugin;

impl Plugin for SchudulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                InGameSet::LoadEntities,
                InGameSet::DespawnEntities,
                // apply_deferred(Flush)
                InGameSet::GameInput,
                InGameSet::UpdateEntities,
                InGameSet::CollisionDetection,
                InGameSet::CollisionReaction,
                InGameSet::RenderSetup,
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
