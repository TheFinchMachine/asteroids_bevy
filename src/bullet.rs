use crate::{bodies::*, schedule::InGameSet, GameState};
use bevy::prelude::*;
use bevy_easy_config::EasyConfigPlugin;
use serde::Deserialize;
use std::time::Duration;

#[derive(Resource, Default, Deserialize, Asset, Clone, Copy, TypePath)]
struct BulletConfig {
    speed: f32,
    lifetime: u64,
    size: f32,
    color: (f32, f32, f32),
}

#[derive(Resource)]
struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct Bullet;

#[derive(Bundle)]
struct BulletBundle {
    bullet: Bullet,
    position: Position,
    rotation: Rotation,
    velocity: Velocity,
    angular_velocity: AngularVelocity,
    scale: Scale,
    spawn_time: TimeStamp,
    rigid_body: RigidBody,
    collider: Collider,
}

impl BulletBundle {
    fn new(position: Vec2, rotation: f32, spawn_time: Duration, speed: f32) -> Self {
        Self {
            bullet: Bullet,
            position: Position(position),
            rotation: Rotation(rotation),
            angular_velocity: AngularVelocity(0.0),
            scale: Scale(1.0),
            velocity: Velocity(Rot2::radians(rotation) * Vec2::new(0.0, speed)),
            spawn_time: TimeStamp(spawn_time),
            rigid_body: RigidBody {
                radius: 0.02,
                mass: 2.0,
            },
            // TODO! inherit team from ship
            collider: Collider { team: 1 },
        }
    }
}

fn load_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<BulletConfig>,
) {
    let shape = Circle::new(config.size);
    let color = Color::srgb(config.color.0, config.color.1, config.color.2);

    let mesh = meshes.add(shape);
    let material = materials.add(color);

    commands.insert_resource(BulletAssets { mesh, material })
}

#[derive(Event, Debug)]
pub struct CreateBullet {
    pub position: Vec2,
    pub rotation: f32,
}

// TODO! switch to spawning bullets with an event
// event chaining is fine, as long as you schedule them correctly
fn spawn_bullet(
    mut commands: Commands,
    mut events: EventReader<CreateBullet>,
    bullet_assets: Res<BulletAssets>,
    time: Res<Time>,
    config: Res<BulletConfig>,
) {
    for event in events.read() {
        commands.spawn((
            BulletBundle::new(event.position, event.rotation, time.elapsed(), config.speed),
            Mesh2d(bullet_assets.mesh.clone()),
            MeshMaterial2d(bullet_assets.material.clone()),
            Transform::default(),
        ));
    }
}

fn destroy_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &TimeStamp), With<Bullet>>,
    time: Res<Time>,
    config: Res<BulletConfig>,
) {
    let time_elapsed = time.elapsed();
    for (entity, spawn_time) in &bullets {
        if time_elapsed - spawn_time.0 > Duration::from_millis(config.lifetime) {
            commands.entity(entity).despawn();
        }
    }
}

fn collisions_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &Collider), With<Bullet>>,
    colliders: Query<(Entity, &Collider)>,
    mut collisions: EventReader<Collision>,
) {
    for event in collisions.read() {
        if let Ok((ship, ship_collider)) = bullets.get(event.entity1) {
            if let Ok((_, collider)) = colliders.get(event.entity2) {
                if collider.team != ship_collider.team {
                    commands.entity(ship).despawn();
                }
            }
        } else if let Ok((ship, ship_collider)) = bullets.get(event.entity2) {
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

fn despawn_bullets(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateBullet>();
        app.add_plugins(EasyConfigPlugin::<BulletConfig>::new("bullet.cfg.ron"));
        app.add_systems(Startup, load_bullet);
        app.add_systems(
            Update,
            (destroy_bullets, collisions_bullets).in_set(InGameSet::DespawnEntities),
        );
        app.add_systems(Update, (spawn_bullet).in_set(InGameSet::CollisionDetection));
        app.add_systems(OnEnter(GameState::GameOver), despawn_bullets);
    }
}
