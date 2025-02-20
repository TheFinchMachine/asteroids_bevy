use crate::schedule::InGameSet;
use bevy::prelude::*;
use std::time::Duration;
//use web_sys::console;


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
pub struct Damping(pub f32);

#[derive(Component)]
pub struct AngularDamping(pub f32);

#[derive(Component)]
pub struct Scale(pub f32);

#[derive(Component)]
pub struct NeedsMesh;

#[derive(Component)]
pub struct NeedsConfig;

#[derive(Component)]
pub struct NeedsMaterial;

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

fn collide(pos1: Vec2, pos2: Vec2, r1: f32, r2: f32) -> (Vec2, f32, f32) {
    let dir = pos2 - pos1;
    let dist = dir.length().abs();
    let collide_dist = r1 + r2;
    (dir, dist, collide_dist)
}

#[derive(Event)]
pub struct Collision {
    pub entity1: Entity,
    pub entity2: Entity,
    pub dir: Vec2,
    pub dist: f32,
    pub collide_dist: f32,
}

// lets call asteroids team 0
#[derive(Component, Debug)]
pub struct Collider {
    pub team: u32,
}

fn collisions(
    mut bodies: Query<(Entity, &Position, &RigidBody)>,
    mut collision_writer: EventWriter<Collision>,
) {
    let mut combinations = bodies.iter_combinations_mut();
    while let Some([(entity1, pos1, body1), (entity2, pos2, body2)]) = combinations.fetch_next() {
        let (dir, dist, collide_dist) = collide(pos1.0, pos2.0, body1.radius, body2.radius);
        if dist < collide_dist {
            //console::log_1(&"Sending CollisionEvent".into());
            collision_writer.send(Collision {
                entity1,
                entity2,
                dir,
                dist,
                collide_dist,
            });
        }
    }
}

fn update_position(time: Res<Time>, mut obj: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut obj {
        position.0 += velocity.0 * time.delta_secs();
    }
}
fn update_rotation(time: Res<Time>, mut obj: Query<(&mut Rotation, &AngularVelocity)>) {
    for (mut rotation, angular_velocity) in obj.iter_mut() {
        rotation.0 += angular_velocity.0 * time.delta_secs();
    }
}
fn update_velocity(time: Res<Time>, mut obj: Query<(&mut Velocity, &Acceleration, &Rotation)>) {
    for (mut velocity, acceleration, rotation) in obj.iter_mut() {
        let rotator = Rot2::radians(rotation.0);
        velocity.0 += rotator * acceleration.0 * time.delta_secs();
    }
}
fn update_angular_velocity(
    time: Res<Time>,
    mut obj: Query<(&mut AngularVelocity, &AngularAcceleration)>,
) {
    for (mut angular_velocity, angular_acceleration) in obj.iter_mut() {
        angular_velocity.0 += angular_acceleration.0 * time.delta_secs();
    }
}
fn damping(time: Res<Time>, mut obj: Query<(&mut Velocity, &Damping)>) {
    for (mut velocity, damping) in obj.iter_mut() {
        velocity.0 *= (-damping.0 * time.delta_secs()).exp();
    }
}
fn damping_angular(time: Res<Time>, mut obj: Query<(&mut AngularVelocity, &AngularDamping)>) {
    for (mut angular_velocity, damping) in obj.iter_mut() {
        angular_velocity.0 *= (-damping.0 * time.delta_secs()).exp();
    }
}

pub struct BodiesPlugin;

impl Plugin for BodiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Collision>();
        app.add_systems(
            Update,
            (update_velocity, update_position, damping)
                .chain()
                .in_set(InGameSet::UpdateEntities),
        );
        app.add_systems(
            Update,
            (update_angular_velocity, update_rotation, damping_angular)
                .chain()
                .in_set(InGameSet::UpdateEntities),
        );
        app.add_systems(Update, (collisions).in_set(InGameSet::CollisionDetection));
    }
}
