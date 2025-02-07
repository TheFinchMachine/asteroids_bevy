use bevy::prelude::*;

#[derive(Event)]
pub struct Accelerate {
    controller: Entity,
    direction: Vec2,
}

#[derive(Event)]
pub struct AccelerateAngular {
    entity: Entity,
    direction: f32,
}

#[derive(Event)]
pub struct Shoot {
    entity: Entity,
}
