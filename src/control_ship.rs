use bevy::prelude::*;

#[derive(Event)]
pub struct Accelerate {
    pub controller: Entity,
    pub direction: Vec2,
}

#[derive(Event)]
pub struct AccelerateAngular {
    pub controller: Entity,
    pub direction: f32,
}

#[derive(Event)]
pub struct Shoot {
    pub controller: Entity,
}
