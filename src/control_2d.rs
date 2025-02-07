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

pub struct Control2dPlugin;

impl Plugin for Control2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Accelerate>();
        app.add_event::<AccelerateAngular>();
        app.add_event::<Shoot>();
    }
}
