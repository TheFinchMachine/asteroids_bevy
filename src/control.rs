use bevy::prelude::*;

//TODO! convert parameters to private
#[derive(Component)]
pub struct PlayerController {
    pub id: u32,
}

impl Controller for PlayerController {
    fn get_id(&self) -> &u32 {
        &self.id
    }
    fn get_mut_id(&mut self) -> &mut u32 {
        &mut self.id
    }
}

trait Controller {
    fn get_id(&self) -> &u32;
    fn get_mut_id(&mut self) -> &mut u32;
}

#[derive(Component)]
pub struct ShipPawn {
    controller: Entity,
}

impl Pawn for ShipPawn {
    fn new(controller: Entity) -> Self {
        Self { controller }
    }
    fn get_controller(&self) -> &Entity {
        &self.controller
    }
    fn get_mut_controller(&mut self) -> &mut Entity {
        &mut self.controller
    }
}

pub trait Pawn {
    fn new(controller: Entity) -> Self;
    fn get_controller(&self) -> &Entity;
    fn get_mut_controller(&mut self) -> &mut Entity;
}
