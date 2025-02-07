use crate::control::PlayerController;
use crate::control_2d::*;
use bevy::prelude::*;

pub fn handle_player_input(
    controllers: Query<(Entity, &PlayerController)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut accel_writer: EventWriter<Accelerate>,
    mut accel_ang_writer: EventWriter<AccelerateAngular>,
    mut shoot_writer: EventWriter<Shoot>,
) {
    for (entity, _controller) in controllers.iter() {
        // TODO! add input handling to route keys to specific players
        let mut acceleration = Accelerate {
            controller: entity,
            direction: Vec2::new(0.0, 0.0),
        };
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            acceleration.direction.y = 1.0;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            acceleration.direction.y = -1.0;
        } else {
            acceleration.direction.y = 0.0;
        }
        accel_writer.send(acceleration);

        let mut accel_angular = AccelerateAngular {
            controller: entity,
            direction: 0.0,
        };
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            accel_angular.direction = -1.0;
        } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
            accel_angular.direction = 1.0;
        } else {
            accel_angular.direction = 0.0;
        }
        accel_ang_writer.send(accel_angular);

        if keyboard_input.pressed(KeyCode::Space) {
            shoot_writer.send(Shoot { controller: entity });
        }
    }
}
