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
use control_ship::Accelerate;
use control_ship::AccelerateAngular;
use control_ship::Shoot;
use schedule::SchudulePlugin;
use std::time::Duration;

mod asteroid;
mod bodies;
mod bullet;
mod control;
mod control_ship;
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
        app.add_plugins(BodiesPlugin);
        app.init_state::<GameState>();
        app.add_event::<Accelerate>();
        app.add_event::<AccelerateAngular>();
        app.add_event::<Shoot>();
        app.add_systems(
            Startup,
            (
                spawn_camera,
                spawn_ship,
                load_spawner,
                load_bullet,
                grid_build,
                load_asteroids.after(load_spawner),
                //spawn_asteroid_random.after(load_asteroids),
            ),
        );
        app.add_systems(Update, pause_system);
        app.add_systems(
            Update,
            (
                handle_player_input,
                spawn_asteroid_random.run_if(on_timer(Duration::from_secs(2))),
                //move_obj,
                //move_ship,
                apply_accel,
                apply_accel_ang,
                shoot,
                wrap_obj,
                on_resize,
                collisions_asteroids,
                collisions_ship,
                collisions_bullets,
                destroy_bullets,
            )
                .in_set(ObjectUpdate)
                .run_if(in_state(GameState::InGame)),
        );
        app.add_systems(Update, project_positions.after(ObjectUpdate));
    }
}
