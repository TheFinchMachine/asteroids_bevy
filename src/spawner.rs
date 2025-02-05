use bevy::prelude::*;
use bevy_turborand::prelude::*;

#[derive(Resource)]
pub struct SpawnGenerator {
    pub rng: RngComponent,
}

pub fn load_spawner(mut commands: Commands, mut global_rng: ResMut<GlobalRng>) {
    commands.insert_resource(SpawnGenerator {
        rng: RngComponent::from(&mut global_rng),
    });
}
