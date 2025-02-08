use crate::asteroid::*;
use crate::bodies::*;
use crate::bullet::*;
use crate::grid::*;
use crate::input::*;
use crate::score::*;
use crate::ship::*;
use crate::spawner::*;
use crate::states::*;
use bevy::prelude::*;

use bevy_turborand::prelude::*;
use control_2d::Control2dPlugin;
use schedule::InGameSet;
use schedule::SchudulePlugin;

mod asteroid;
mod bodies;
mod bullet;
mod control;
mod control_2d;
mod grid;
mod input;
mod schedule;
mod score;
mod ship;
mod spawner;
mod states;

const WORLD_SEED: u64 = 1024;

pub struct AsteroidsGamePlugin;

impl Plugin for AsteroidsGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::new().with_rng_seed(WORLD_SEED));
        app.add_plugins(ScorePlugin);
        app.add_plugins(SchudulePlugin);
        app.add_plugins(ShipPlugin);
        app.add_plugins(BodiesPlugin);
        app.add_plugins(Control2dPlugin);
        app.add_plugins(StatePlugin);
        app.add_plugins(GridPlugin);
        app.add_plugins(BulletPlugin);
        app.add_plugins(AsteroidsPlugin);

        app.add_systems(Update, (handle_player_input,).in_set(InGameSet::GameInput));
    }
}
