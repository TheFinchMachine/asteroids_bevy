use crate::bodies::*;
use bevy::prelude::*;
use std::time::Duration;

const BULLET_SPEED: f32 = 6.0;
const BULLET_LIFETIME: Duration = Duration::from_millis(500);

#[derive(Resource)]
pub struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct Bullet;

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
}

impl BulletBundle {
    fn new(position: Vec2, rotation: f32, spawn_time: Duration) -> Self {
        Self {
            bullet: Bullet,
            position: Position(position),
            rotation: Rotation(rotation),
            angular_velocity: AngularVelocity(0.0),
            scale: Scale(1.0),
            velocity: Velocity(Rot2::radians(rotation) * Vec2::new(0.0, BULLET_SPEED)),
            spawn_time: TimeStamp(spawn_time),
            rigid_body: RigidBody {
                radius: 0.02,
                mass: 2.0,
            },
        }
    }
}

pub fn load_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle::new(4.);
    let color = Color::srgb(1., 0., 0.);

    let mesh = meshes.add(shape);
    let material = materials.add(color);

    commands.insert_resource(BulletAssets { mesh, material })
}

pub fn spawn_bullet(
    commands: &mut Commands,
    bullet_assets: &Res<BulletAssets>,
    position: Vec2,
    rotation: f32,
    time: &Res<Time>,
) {
    commands.spawn((
        BulletBundle::new(position, rotation, time.elapsed()),
        Mesh2d(bullet_assets.mesh.clone()),
        MeshMaterial2d(bullet_assets.material.clone()),
        Transform::default(),
    ));
}

pub fn destroy_bullets(
    bullets: Query<(Entity, &TimeStamp), With<Bullet>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let time_elapsed = time.elapsed();
    for (entity, spawn_time) in &bullets {
        if time_elapsed - spawn_time.0 > BULLET_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}
