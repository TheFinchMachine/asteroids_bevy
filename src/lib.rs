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
use bevy::time::common_conditions::on_timer;
use bevy_turborand::prelude::*;
use control_2d::Control2dPlugin;
use schedule::SchudulePlugin;
use std::time::Duration;

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

pub struct AsteroidsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ObjectUpdate;

impl Plugin for AsteroidsPlugin {
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

        app.add_systems(
            Startup,
            (
                load_spawner,
                load_asteroids.after(load_spawner),
                //spawn_asteroid_random.after(load_asteroids),
            ),
        );
        app.add_systems(
            Update,
            (
                handle_player_input,
                spawn_asteroid_random.run_if(on_timer(Duration::from_secs(2))),
                collisions_asteroids,
                collisions_ship,
                collisions_bullets,
            )
                .in_set(ObjectUpdate)
                .run_if(in_state(GameState::InGame)),
        );
    }
}
