use bevy::prelude::*;

#[derive(Component)]
pub struct Controller;

#[derive(Component)]
pub struct Pawn {
    controller: Entity,
}
