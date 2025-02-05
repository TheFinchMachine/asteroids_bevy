use crate::bodies::*;
use crate::bullet::*;
use crate::ship::*;

use bevy::prelude::*;
use std::time::Duration;

const SHOT_SPACING: Duration = Duration::from_millis(350);

pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bullet_assets: Res<BulletAssets>,
    time: Res<Time>,
    commands: Commands,
    mut ship: Query<
        (
            &mut Acceleration,
            &mut AngularAcceleration,
            &Position,
            &Rotation,
            &mut TimeStamp,
        ),
        With<Ship>,
    >,
) {
    if let Ok((
        mut acceleration,
        mut angular_acceleration,
        position,
        rotation,
        mut last_shot_time,
    )) = ship.get_single_mut()
    {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            acceleration.0.y = SHIP_SPEED;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            acceleration.0.y = -SHIP_SPEED;
        } else {
            acceleration.0.y = 0.;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            angular_acceleration.0 = -SHIP_SPEED_ANGULAR;
        } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
            angular_acceleration.0 = SHIP_SPEED_ANGULAR;
        } else {
            angular_acceleration.0 = 0.;
        }

        if keyboard_input.pressed(KeyCode::Space) {
            let time_elapsed = time.elapsed();
            if time_elapsed - last_shot_time.0 > SHOT_SPACING {
                spawn_bullet(commands, bullet_assets, position.0, rotation.0, time);
                last_shot_time.0 = time_elapsed;
            }
        }
    }
}
