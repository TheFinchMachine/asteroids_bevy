use bevy::input::keyboard::Key;
use bevy::{prelude::*, scene};
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::time::Duration;

#[derive(Component)]
struct TimeStamp(Duration);

// don't use Rot2 as it is effectivly a 2d quat. 2d rots don't suffer from gimbal lock, so we don't need that complexity.
#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct AngularVelocity(f32);

#[derive(Component)]
struct AngularAcceleration(f32);

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Shape(Vec2);

const SHIP_SPEED: f32 = 0.3;
const SHIP_DAMPING: f32 = 0.99;

const SHIP_SPEED_ANGULAR: f32 = 0.02;
const SHIP_DAMPING_ANGULAR: f32 = 0.8;

#[derive(Component)]
struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    shape: Shape,
    position: Position,
    velocity: Velocity,
    acceleration: Acceleration,
    rotation: Rotation,
    angular_velocity: AngularVelocity,
    angular_acceleration: AngularAcceleration,
}

impl ShipBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ship: Ship,
            shape: Shape(Vec2::new(10., 10.)),
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::new(0., 0.)),
            acceleration: Acceleration(Vec2::new(0., 0.)),
            rotation: Rotation(0.0),
            angular_velocity: AngularVelocity(0.0),
            angular_acceleration: AngularAcceleration(0.0),
        }
    }
}

fn spawn_ship(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>, 
    asset_server: Res<AssetServer>,
) {
    let mesh = asset_server.load(
        GltfAssetLabel::Primitive { mesh: 0, primitive: 0 }.from_asset("meshes/ship.gltf")
    );

    let color = Color::srgb(0.8, 0.8, 1.0);
    let material = materials.add(color);

    commands.spawn((
        ShipBundle::new(0., 0.),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_scale(Vec3::new(10., 10., 10.)),
    ));
}

fn move_ship(
    mut ship: Query<(
        &mut Position, &mut Velocity, &Acceleration,
        &mut Rotation, &mut AngularVelocity, &AngularAcceleration), With<Ship>>
) {
    for (mut position, mut velocity, acceleration,
        mut rotation, mut angular_velocity, angular_acceleration,) in &mut ship {
        angular_velocity.0 += angular_acceleration.0 * SHIP_SPEED_ANGULAR;
        rotation.0 += angular_velocity.0;
        angular_velocity.0 *= SHIP_DAMPING_ANGULAR;

        let rotator = Rot2::radians(rotation.0);

        velocity.0 += rotator * acceleration.0  * SHIP_SPEED;
        position.0 += velocity.0;
        velocity.0 *= SHIP_DAMPING;
    }
}

const BULLET_SPEED: f32 = 10.0;
const BULLET_LIFETIME: Duration = Duration::from_millis(500);

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
    shape: Shape,
    position: Position,
    rotation: Rotation,
    velocity: Velocity,
    spawn_time: TimeStamp,
}

impl BulletBundle {
    fn new(position: Vec2, rotation: f32, spawn_time: Duration) -> Self {
        Self {
            bullet: Bullet,
            shape: Shape(Vec2::new(4., 4.)),
            position: Position(position),
            rotation: Rotation(rotation),
            velocity: Velocity(Rot2::radians(rotation) * Vec2::new(0.0, BULLET_SPEED)),
            spawn_time: TimeStamp(spawn_time),
        }
    }
}

fn load_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle::new(4.);
    let color = Color::srgb(1., 0., 0.);

    let mesh = meshes.add(shape);
    let material = materials.add(color);

    commands.insert_resource(BulletAssets {
        mesh,
        material
    })
}

// TODO: use the same mesh and material for all bullets
fn spawn_bullet(
    mut commands: Commands,
    bullet_assets: Res<BulletAssets>,
    position: Vec2,
    rotation: f32,
    time: Res<Time>,
) { 
    commands.spawn((
        BulletBundle::new(position, rotation, time.elapsed()),
        Mesh2d(bullet_assets.mesh.clone().into()),
        MeshMaterial2d(bullet_assets.material.clone()),
        Transform::default()
    ));
}

// TODO; calculate velocity vector on spawn
fn move_bullets(
    mut bullets: Query<(&mut Position, &Velocity), With<Bullet>>,
) {
    for (mut position, velocity) in &mut bullets {
        position.0 += velocity.0;
    }
}

fn destroy_bullets (
    bullets: Query<(Entity, &TimeStamp), With<Bullet>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let time_elapsed = time.elapsed();
    for (entity, spawn_time) in &bullets { 
        if time_elapsed - spawn_time.0 > BULLET_LIFETIME  {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(Camera2d);
}

const GRID_SIZE: f32 = 1.;
fn project_positions(
    mut positionables: Query<(&mut Transform, &Position, &Rotation)>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let window_width = window.resolution.width();

        //let window_aspect = window_width / window_height;

        for (mut transform, position, rotation) in &mut positionables {
            let mut new_position = position.0;
            // Do we want to scale to window so multiple players will see the same thing?
            // or keep the positions consistent on an absolute and just consider the wraparound to be a projection.
            // wraparound as projection is a nice visual, but makes collision strange.
            new_position *= GRID_SIZE;
            //wrap objects around the screen
            new_position.x = wrap_around(new_position.x, -window_width/2., window_width);
            new_position.y = wrap_around(new_position.y, -window_height/2., window_height);
            //println!("new_position.y: {}", new_position.y);
            transform.translation = new_position.extend(0.);

            transform.rotation = Quat::from_rotation_z(rotation.0);
        }
    }
}

fn wrap_around(value: f32, min_value: f32, range: f32) -> f32 {
    // modulo preserves sign so we need to add range and then modulo again to handle negatives
    // could also be done with an if statement but this is specifically branchless
    ((value - min_value) % range + range) % range + min_value
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bullet_assets: Res<BulletAssets>,
    time: Res<Time>,
    mut commands: Commands,
    mut ship: Query<(&mut Acceleration, &mut AngularAcceleration, &Position, &Rotation), With<Ship>>,
) {
    if let Ok((mut acceleration, mut angular_acceleration, position, rotation)) = ship.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            acceleration.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            acceleration.0.y = -1.;
        } else {
            acceleration.0.y = 0.;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            angular_acceleration.0 = -1.;
        } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
            angular_acceleration.0 = 1.;
        } else {
            angular_acceleration.0 = 0.;
        }

        if keyboard_input.pressed(KeyCode::Space) {
            spawn_bullet(
                commands,
                bullet_assets,
                position.0,
                rotation.0,
                time);
        }
    }
}

pub struct AsteroidsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ObjectUpdate;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            spawn_camera,
            spawn_ship,
            load_bullet,
        ));
        app.add_systems(Update, (
            handle_player_input,
            move_ship,
            move_bullets,
            destroy_bullets,
        ).in_set(ObjectUpdate));
        app.add_systems(Update, (
            project_positions.after(ObjectUpdate)
        ));
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(AsteroidsPlugin)
    .run();
}