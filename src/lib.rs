use crate::asteroid::*;
use crate::bodies::*;
use crate::bullet::*;
use crate::grid::*;
use crate::input::*;
use crate::score::*;
use crate::ship::*;
use crate::spawner::*;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_turborand::prelude::*;
use std::time::Duration;

mod asteroid;
mod bodies;
mod bullet;
mod grid;
mod input;
mod score;
mod ship;
mod spawner;

const WORLD_SEED: u64 = 1024;

pub struct AsteroidsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ObjectUpdate;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::new().with_rng_seed(WORLD_SEED));
        app.init_resource::<Score>();
        app.add_event::<Scored>();
        app.add_systems(
            Startup,
            (
                spawn_camera,
                spawn_ship,
                spawn_scoreboard,
                load_spawner,
                load_bullet,
                grid_build,
                load_asteroids.after(load_spawner),
                //spawn_asteroid_random.after(load_asteroids),
            ),
        );
        app.add_systems(
            Update,
            (
                handle_player_input,
                spawn_asteroid_random.run_if(on_timer(Duration::from_secs(2))),
                move_obj,
                move_ship,
                wrap_obj,
                on_resize,
                collisions_asteroids,
                collisions_ship,
                collisions_bullets,
                destroy_bullets,
                update_score.after(collisions_bullets),
                update_scoreboard.after(collisions_bullets),
            )
                .in_set(ObjectUpdate),
        );
        app.add_systems(Update, project_positions.after(ObjectUpdate));
    }
}
