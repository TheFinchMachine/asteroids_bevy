use crate::asteroid::*;
use crate::bullet::Bullet;
use crate::score::Scored;
use crate::ship::Ship;
use crate::spawner::SpawnGenerator;
use bevy::prelude::*;
use bevy_turborand::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct TimeStamp(pub Duration);

// don't use Rot2 as it is effectively a 2d quat. 2d rots don't suffer from gimbal lock, so we don't need that complexity.
#[derive(Component)]
pub struct Rotation(pub f32);

#[derive(Component)]
pub struct AngularVelocity(pub f32);

#[derive(Component)]
pub struct AngularAcceleration(pub f32);

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Acceleration(pub Vec2);

#[derive(Component)]
pub struct Scale(pub f32);

#[derive(Component)]
pub struct RigidBody {
    pub radius: f32,
    pub mass: f32,
}

pub fn collision_bounce(
    vel1: Vec2,
    vel2: Vec2,
    normal: Vec2,
    mass1: f32,
    mass2: f32,
) -> (Vec2, Vec2) {
    let tangent = Vec2::new(-normal.y, normal.x);

    let vel1_normal = vel1.dot(normal);
    let vel1_tangent = vel1.dot(tangent);
    let vel2_normal = vel2.dot(normal);
    let vel2_tangent = vel2.dot(tangent);

    let vel1_normal_new =
        (vel1_normal * (mass1 - mass2) + 2.0 * mass2 * vel2_normal) / (mass1 + mass2);
    let vel2_normal_new =
        (vel2_normal * (mass2 - mass1) + 2.0 * mass1 * vel1_normal) / (mass1 + mass2);

    (
        (tangent * vel1_tangent) + (normal * vel1_normal_new),
        (tangent * vel2_tangent) + (normal * vel2_normal_new),
    )
}

pub fn collide(pos1: Vec2, pos2: Vec2, r1: f32, r2: f32) -> (Vec2, f32, f32) {
    let dir = pos2 - pos1;
    let dist = dir.length().abs();
    let collide_dist = r1 + r2;
    (dir, dist, collide_dist)
}

pub fn collisions_asteroids(
    mut bodies: Query<(&mut Position, &mut Velocity, &RigidBody), With<Asteroid>>,
) {
    let mut combinations = bodies.iter_combinations_mut();
    while let Some([(mut pos1, mut vel1, body1), (mut pos2, mut vel2, body2)]) =
        combinations.fetch_next()
    {
        let (dir, dist, collide_dist) = collide(pos1.0, pos2.0, body1.radius, body2.radius);

        if dist < collide_dist {
            let normal = dir.normalize();
            (vel1.0, vel2.0) = collision_bounce(vel1.0, vel2.0, normal, body1.mass, body2.mass);

            let depth = collide_dist - dist;
            let correction = normal * (depth * 0.5);
            pos1.0 -= correction;
            pos2.0 += correction;
        }
    }
}

pub fn collisions_ship(
    mut commands: Commands,
    ships: Query<(Entity, &Position, &RigidBody), With<Ship>>,
    asteroids: Query<(&Position, &RigidBody), With<Asteroid>>,
) {
    for (ship_entity, ship_pos, ship_body) in &ships {
        for (ast_pos, ast_body) in &asteroids {
            let (_, dist, collide_dist) =
                collide(ship_pos.0, ast_pos.0, ship_body.radius, ast_body.radius);
            if dist < collide_dist {
                commands.entity(ship_entity).despawn();
            }
        }
    }
}

pub fn collisions_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &Position, &RigidBody), With<Bullet>>,
    asteroids: Query<(Entity, &Position, &Velocity, &Scale, &RigidBody), With<Asteroid>>,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
    mut events: EventWriter<Scored>,
) {
    for (bul_entity, bul_pos, bul_body) in &bullets {
        for (ast_entity, ast_pos, ast_vel, ast_scale, ast_body) in &asteroids {
            let (_, dist, collide_dist) =
                collide(bul_pos.0, ast_pos.0, bul_body.radius, ast_body.radius);
            if dist < collide_dist {
                events.send(Scored);
                commands.entity(bul_entity).despawn();
                commands.entity(ast_entity).despawn();
                let av1 = spawner.rng.f32_normalized();
                let av2 = spawner.rng.f32_normalized();

                if ast_scale.0 > 25.0 {
                    spawn_asteroid_child(
                        &mut commands,
                        &asteroid_assets,
                        &mut spawner,
                        ast_pos.0,
                        ast_vel.0,
                        av1,
                        ast_scale.0,
                        50.0,
                    );
                    spawn_asteroid_child(
                        &mut commands,
                        &asteroid_assets,
                        &mut spawner,
                        ast_pos.0,
                        ast_vel.0,
                        av2,
                        ast_scale.0,
                        -50.0,
                    );
                }
            }
        }
    }
}

pub fn move_obj(
    time: Res<Time>,
    mut obj: Query<(&mut Position, &mut Rotation, &Velocity, &AngularVelocity), Without<Ship>>,
) {
    for (mut position, mut rotation, velocity, angular_velocity) in &mut obj {
        position.0 += velocity.0 * time.delta_secs();
        rotation.0 += angular_velocity.0 * time.delta_secs();
    }
}
