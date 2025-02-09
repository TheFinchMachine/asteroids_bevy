use crate::{
    bodies::*,
    bullet::CreateBullet,
    control::{Pawn, PlayerController, ShipPawn},
    control_2d::{Accelerate, AccelerateAngular, Shoot},
    schedule::InGameSet,
    GameState,
};
use bevy::prelude::*;
use bevy_easy_config::EasyConfigPlugin;
use serde::Deserialize;
use std::time::Duration;

#[derive(Resource, Default, Deserialize, Asset, Clone, TypePath)]
struct ShipConfig {
    speed: f32,
    damping: f32,
    speed_angular: f32,
    damping_angular: f32,
    asset_path: String,
    color: (f32, f32, f32),
    fire_delay: u64,
    fire_reload: u64,
    fire_magazine: u32,
}

#[derive(Component)]
pub struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    pawn: ShipPawn,
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
    fn new(x: f32, y: f32, pawn: ShipPawn, damping: f32, angular_damping: f32) -> Self {
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
    config: Res<ShipConfig>,
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
        .from_asset(config.asset_path.clone()),
    );

    let color = Color::srgb(config.color.0, config.color.1, config.color.2);
    let material = materials.add(color);

    commands.spawn((
        ShipBundle::new(
            0.,
            0.,
            ShipPawn::new(player_entity),
            config.damping,
            config.damping_angular,
        ),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
    ));
}

fn apply_accel(
    config: Res<ShipConfig>,
    mut ships: Query<(&mut Acceleration, &ShipPawn), With<Ship>>,
    mut events: EventReader<Accelerate>,
) {
    for event in events.read() {
        for (mut acceleration, pawn) in ships.iter_mut() {
            if pawn.get_controller() == &event.controller {
                acceleration.0 = config.speed * event.direction;
            }
        }
    }
}

fn apply_accel_ang(
    config: Res<ShipConfig>,
    mut ships: Query<(&mut AngularAcceleration, &ShipPawn), With<Ship>>,
    mut events: EventReader<AccelerateAngular>,
) {
    for event in events.read() {
        for (mut angular_accel, pawn) in ships.iter_mut() {
            if pawn.get_controller() == &event.controller {
                angular_accel.0 = config.speed_angular * event.direction;
            }
        }
    }
}

//TODO! add magazine
fn shoot(
    time: Res<Time>,
    mut ships: Query<(&Position, &Rotation, &mut TimeStamp, &ShipPawn), With<Ship>>,
    mut events: EventReader<Shoot>,
    mut create_bullet: EventWriter<CreateBullet>,
    config: Res<ShipConfig>,
) {
    for event in events.read() {
        for (position, rotation, mut last_shot_time, pawn) in ships.iter_mut() {
            if pawn.get_controller() == &event.controller {
                let time_elapsed = time.elapsed();
                if time_elapsed - last_shot_time.0 > Duration::from_millis(config.fire_delay) {
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
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in collisions.read() {
        for (entity_a, entity_b) in [
            (event.entity1, event.entity2),
            (event.entity2, event.entity1),
        ] {
            if let Ok((ship, ship_collider)) = ships.get(entity_a) {
                if let Ok((_, collider)) = colliders.get(entity_b) {
                    if collider.team != ship_collider.team {
                        commands.entity(ship).despawn();
                        next_state.set(GameState::GameOver);
                    }
                }
            }
        }
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EasyConfigPlugin::<ShipConfig>::new("ship.cfg.ron"));
        app.add_systems(OnEnter(GameState::InGame), (spawn_ship));
        app.add_systems(
            Update,
            (apply_accel, apply_accel_ang, shoot).in_set(InGameSet::EntityUpdates),
        );
        app.add_systems(Update, (collisions_ship).in_set(InGameSet::DespawnEntities));
    }
}
