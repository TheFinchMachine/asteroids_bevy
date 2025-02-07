use bevy::prelude::*;

#[derive(Component)]
pub struct Controller {
    id: u32,
}

#[derive(Component)]
pub struct PlayerController {
    pub id: u32,
}

#[derive(Component)]
pub struct Pawn {
    pub controller: Entity,
}
