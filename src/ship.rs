use crate::{
    bodies::*,
    bullet::spawn_bullet,
    control::{Pawn, PlayerController},
    control_ship::{Accelerate, AccelerateAngular, Shoot},
    BulletAssets,
};
use bevy::prelude::*;
use std::time::Duration;

pub const SHIP_SPEED: f32 = 2.0;
const SHIP_DAMPING: f32 = 1.0;

pub const SHIP_SPEED_ANGULAR: f32 = 6.0;
const SHIP_DAMPING_ANGULAR: f32 = 10.0;

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
}

impl ShipBundle {
    fn new(x: f32, y: f32, pawn: Pawn) -> Self {
        Self {
            ship: Ship,
            pawn,
            position: Position(Vec2::new(x, y)),
            rotation: Rotation(0.0),
            scale: Scale(10.0),
            velocity: Velocity(Vec2::new(0., 0.)),
            acceleration: Acceleration(Vec2::new(0., 0.)),
            damping: Damping(SHIP_DAMPING),
            angular_velocity: AngularVelocity(0.0),
            angular_acceleration: AngularAcceleration(0.0),
            angular_damping: AngularDamping(SHIP_DAMPING_ANGULAR),
            last_shot: TimeStamp(Duration::ZERO),
            rigid_body: RigidBody {
                radius: 0.1,
                mass: 2.0,
            },
        }
    }
}

pub fn spawn_ship(
    mut commands: Commands,
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
        ),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
    ));
}

pub fn apply_accel(
    mut ships: Query<(&mut Acceleration, &Pawn), With<Ship>>,
    mut events: EventReader<Accelerate>,
) {
    for event in events.read() {
        for (mut acceleration, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                acceleration.0 = SHIP_SPEED * event.direction;
            }
        }
    }
}

pub fn apply_accel_ang(
    mut ships: Query<(&mut AngularAcceleration, &Pawn), With<Ship>>,
    mut events: EventReader<AccelerateAngular>,
) {
    for event in events.read() {
        for (mut angular_accel, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                angular_accel.0 = SHIP_SPEED_ANGULAR * event.direction;
            }
        }
    }
}

const SHOT_SPACING: Duration = Duration::from_millis(350);
pub fn shoot(
    time: Res<Time>,
    bullet_assets: Res<BulletAssets>,
    mut commands: Commands,
    mut ships: Query<(&Position, &Rotation, &mut TimeStamp, &Pawn), With<Ship>>,
    mut events: EventReader<Shoot>,
) {
    for event in events.read() {
        for (position, rotation, mut last_shot_time, pawn) in ships.iter_mut() {
            if pawn.controller == event.controller {
                let time_elapsed = time.elapsed();
                if time_elapsed - last_shot_time.0 > SHOT_SPACING {
                    spawn_bullet(&mut commands, &bullet_assets, position.0, rotation.0, &time);
                    last_shot_time.0 = time_elapsed;
                }
            }
        }
    }
}
