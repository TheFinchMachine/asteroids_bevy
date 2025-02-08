use crate::{
    bodies::*,
    bullet::CreateBullet,
    control::{Pawn, PlayerController},
    control_2d::{Accelerate, AccelerateAngular, Shoot},
    schedule::InGameSet,
};
use bevy::prelude::*;
use bevy_easy_config::EasyConfigPlugin;
use serde::Deserialize;
use std::time::Duration;

#[derive(Resource, Default, Deserialize, Asset, Clone, Copy, TypePath)]
struct ShipConfig {
    speed: f32,
    damping: f32,
    speed_angular: f32,
    damping_angular: f32,
}

#[derive(Component)]
pub struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    pawn: Pawn,
    position: Position,
    rotation: Rotation,
    scale: Scale,
    velocity: Velocity,
    acceleration: Acceleration,
    damping: Damping,
    angular_velocity: AngularVelocity,
    angular_acceleration: AngularAcceleration,
    angular_damping: AngularDamping,
    last_shot: TimeStamp,
    rigid_body: RigidBody,
    collider: Collider,
}

impl ShipBundle {
    fn new(x: f32, y: f32, pawn: Pawn, damping: f32, angular_damping: f32) -> Self {
        Self {
            ship: Ship,
            pawn,
            position: Position(Vec2::new(x, y)),
            rotation: Rotation(0.0),
            scale: Scale(10.0),
            velocity: Velocity(Vec2::new(0., 0.)),
            acceleration: Acceleration(Vec2::new(0., 0.)),
            damping: Damping(damping),
            angular_velocity: AngularVelocity(0.0),
            angular_acceleration: AngularAcceleration(0.0),
            angular_damping: AngularDamping(angular_damping),
            last_shot: TimeStamp(Duration::ZERO),
            rigid_body: RigidBody {
                radius: 0.1,
                mass: 2.0,
            },
            collider: Collider { team: 1 },
        }
    }
}

fn spawn_ship(
    mut commands: Commands,
    ship_config: Res<ShipConfig>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    //TODO! move player spawn somewhere sensible
    let player_entity = commands.spawn(PlayerController { id: 0 }).id();
    let mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: 0,
            primitive: 0,
        }
        .from_asset("meshes/ship.glb"),
    );

    let color = Color::srgb(0.8, 0.8, 1.0);
    let material = materials.add(color);

    commands.spawn((
        ShipBundle::new(
            0.,
            0.,
            Pawn {
                controller: player_entity,
            },
            ship_config.damping,
            ship_config.damping_angular,
        ),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
    ));
}

fn apply_accel(
    ship_config: Res<ShipConfig>,
    mut ships: Query<(&mut Acceleration, &Pawn), With<Ship>>,
    mut events: EventReader<Accelerate>,
) {
    for event in events.read() {
        for (mut acceleration, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                acceleration.0 = ship_config.speed * event.direction;
            }
        }
    }
}

fn apply_accel_ang(
    ship_config: Res<ShipConfig>,
    mut ships: Query<(&mut AngularAcceleration, &Pawn), With<Ship>>,
    mut events: EventReader<AccelerateAngular>,
) {
    for event in events.read() {
        for (mut angular_accel, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                angular_accel.0 = ship_config.speed_angular * event.direction;
            }
        }
    }
}

// TODO! switch to spawning bullets with an event.
// event chaining is fine, as long as you schedule them correctly
const SHOT_SPACING: Duration = Duration::from_millis(350);
pub fn shoot(
    time: Res<Time>,
    mut ships: Query<(&Position, &Rotation, &mut TimeStamp, &Pawn), With<Ship>>,
    mut events: EventReader<Shoot>,
    mut create_bullet: EventWriter<CreateBullet>,
) {
    for event in events.read() {
        for (position, rotation, mut last_shot_time, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                let time_elapsed = time.elapsed();
                if time_elapsed - last_shot_time.0 > SHOT_SPACING {
                    create_bullet.send(CreateBullet {
                        position: position.0,
                        rotation: rotation.0,
                    });
                    last_shot_time.0 = time_elapsed;
                }
            }
        }
    }
}

fn collisions_ship(
    mut commands: Commands,
    ships: Query<(Entity, &Collider), With<Ship>>,
    colliders: Query<(Entity, &Collider)>,
    mut collisions: EventReader<Collision>,
) {
    for event in collisions.read() {
        if let Ok((ship, ship_collider)) = ships.get(event.entity1) {
            if let Ok((_, collider)) = colliders.get(event.entity2) {
                if collider.team != ship_collider.team {
                    commands.entity(ship).despawn();
                }
            }
        } else if let Ok((ship, ship_collider)) = ships.get(event.entity2) {
            if let Ok((_, collider)) = colliders.get(event.entity1) {
                if collider.team != ship_collider.team {
                    commands.entity(ship).despawn();
                }
            }
        } else {
            continue;
        }
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EasyConfigPlugin::<ShipConfig>::new("ship.cfg.ron"));
        app.add_systems(Startup, (spawn_ship,));
        app.add_systems(
            Update,
            (apply_accel, apply_accel_ang, shoot).in_set(InGameSet::EntityUpdates),
        );
        app.add_systems(Update, (collisions_ship).in_set(InGameSet::DespawnEntities));
    }
}
