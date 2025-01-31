use bevy::input::keyboard::Key;
use bevy::{prelude::*, scene};
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::time::common_conditions::on_timer;
use bevy::sprite::{Wireframe2dPlugin, Wireframe2dConfig, Wireframe2d};
use std::time::Duration;
use bevy_turborand::prelude::*;


const WORLD_SEED: u64 = 1024;

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
    last_shot: TimeStamp,
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
            last_shot: TimeStamp(Duration::ZERO)
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

const BULLET_SPEED: f32 = 15.0;
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

fn spawn_bullet(
    mut commands: Commands,
    bullet_assets: Res<BulletAssets>,
    position: Vec2,
    rotation: f32,
    time: Res<Time>,
) { 
    commands.spawn((
        BulletBundle::new(position, rotation, time.elapsed()),
        Mesh2d(bullet_assets.mesh.clone()),
        MeshMaterial2d(bullet_assets.material.clone()),
        Transform::default()
    ));
}

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

#[derive(Resource)]
struct SpawnGenerator {
    rng: RngComponent,
}

fn load_spawner(
    mut commands: Commands,
    mut global_rng: ResMut<GlobalRng>,
) {
    commands.insert_resource(SpawnGenerator {
        rng: RngComponent::from(&mut global_rng),
    });
}

const ASTEROID_VARIANTS: usize = 10;

#[derive(Resource)]
struct AsteroidAssets {
    meshes: [Handle<Mesh>; ASTEROID_VARIANTS],
    material: Handle<ColorMaterial>
}

#[derive(Component)]
struct Asteroid;

// TODO: add scale. Needs update to project positions
#[derive(Bundle)]
struct AsteroidBundle {
    asteroid: Asteroid,
    position: Position,
    rotation: Rotation,
    velocity: Velocity,
    angular_velocity: AngularVelocity,
}

// TODO: angular_velocity seeded random
impl AsteroidBundle {
    fn new(position: Vec2, velocity: Vec2) -> Self {
        Self {
            asteroid: Asteroid,
            position: Position(position),
            rotation: Rotation(0.0),
            velocity: Velocity(velocity),
            angular_velocity: AngularVelocity(0.0)
        }
    }
}

fn load_asteroids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawner: ResMut<SpawnGenerator>
) {
    let material = materials.add(Color::srgb(0.5, 1., 0.5));

    let new_meshes: [Handle<Mesh>; ASTEROID_VARIANTS] = std::array::from_fn(|_| {
        meshes.add(create_astroid_mesh(&mut spawner))
    });

    commands.insert_resource(AsteroidAssets {
        meshes: new_meshes,
        material
    });
}

fn spawn_asteroid(
    commands: &mut Commands,
    asteroid_assets: &Res<AsteroidAssets>,
    spawner: &mut ResMut<SpawnGenerator>,
    position: Vec2,
    velocity: Vec2,
) {
    let mesh = spawner.rng.usize(0..ASTEROID_VARIANTS);
    commands.spawn((
        AsteroidBundle::new(position, velocity),
        Mesh2d(asteroid_assets.meshes[mesh].clone()),
        MeshMaterial2d(asteroid_assets.material.clone()),
        Transform::from_scale(Vec3::new(10., 10., 10.)),
    ));
}

//TODO: use a grid size that I can select positions without worrying about window size
//TODO: use a shared seed so we don't get the same velocity every time
fn spawn_asteroid_random(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
) {
    for _ in 0..ASTEROID_VARIANTS{
        let position = Vec2::new(spawner.rng.f32_normalized()*200.0, spawner.rng.f32_normalized()*200.0);
        let velocity = Vec2::new(spawner.rng.f32_normalized()*3.0, spawner.rng.f32_normalized()*3.0);

        spawn_asteroid(&mut commands, &asteroid_assets, &mut spawner, position, velocity);
    }
}

fn move_asteroids(
    mut asteroids: Query<(&mut Position, &mut Rotation, &Velocity, &AngularVelocity), With<Asteroid>>
) {
    for (mut position, mut rotation, velocity, angular_velocity) in &mut asteroids {
        position.0 += velocity.0;
        rotation.0 += angular_velocity.0;
    }
}

fn create_astroid_mesh(spawner: &mut ResMut<SpawnGenerator>) -> Mesh {
    let rng = &mut spawner.rng;
    // create semi-random circle
    let num_verts = rng.usize(8..12);
    let angle_step = 360.0/num_verts as f32;
    let angle_range = angle_step * 0.2;
    let mut positions = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let radius = rng.f32()*0.5 + 1.0;
        let angle = rng.f32_normalized()*(i as f32 * angle_range) + (i as f32 * angle_step);
        let rotator = Rot2::degrees(angle);
        let point = rotator * Vec2::new(0.0, radius);
        positions.push(point)
    }

    // calculate normals for inset
    // normals can face the wrong way if the verts are concave
    // therefore, base normal direction on angle step
    let mut normals = Vec::with_capacity(num_verts);
    let mut cycle = positions.iter().cycle().take(positions.len() + 2);

    let mut previous_position = cycle.next().unwrap();
    let mut current_position = cycle.next().unwrap();
    for next_position in cycle {
        let edge0 = (previous_position - current_position).normalize();
        let edge1 = (next_position - current_position).normalize();
        let mut normal = Vec2::new(0.0, 1.0);
        if edge0.dot(edge1) < -0.99 {  
            normal = Vec2::new(-edge0.y, edge0.x);
        } else {
            normal = (edge0 + edge1).normalize();
        }
        
        if normal.dot(current_position.normalize()) < 0.0 {
            normal = -normal;
        }
        // WARNING: normals offset by one to the left for positions!
        normals.push(normal);
        previous_position = current_position;
        current_position = next_position;
    }
    normals.rotate_right(1);

    // inset
    let mut positions_inset = Vec::with_capacity(num_verts);
    for i in 0..num_verts {
        
        let new_position = positions[i] + (normals[i] * 0.2);
        positions_inset.push(new_position);
    }
    positions.extend(positions_inset);
    let positions_3d: Vec<Vec3> = positions.into_iter().map(|pos| pos.extend(0.0)).collect();

    // calculate triangle indices
    let mut indices = Vec::new();
    for i in 0..num_verts {
        let max = num_verts*2;
        //triangle 1 cw, which is wrong
        indices.push((i % max) as u32);
        indices.push(((i + num_verts) % max) as u32);
        indices.push(((i + 1) % num_verts) as u32);
        
        //triangle 2 cw, which is wrong
        indices.push(((i + num_verts) % max) as u32);
        indices.push(((i + 1) % num_verts + num_verts) as u32);
        indices.push(((i + 1) % num_verts) as u32);
    }

    let normals_3d = vec![[0.0, 0.0, 1.0]; num_verts*2];

    // build mesh
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions_3d)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals_3d)
        .with_inserted_indices(mesh::Indices::U32(indices))
    
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(Camera2d);
}


// TODO: grid should extend a little be beyond the window to avoid pop-in
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

const SHOT_SPACING: Duration = Duration::from_millis(250);

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bullet_assets: Res<BulletAssets>,
    time: Res<Time>,
    mut commands: Commands,
    mut ship: Query<(&mut Acceleration, &mut AngularAcceleration, &Position, &Rotation, &mut TimeStamp), With<Ship>>,
) {
    if let Ok((mut acceleration, mut angular_acceleration, position, rotation, mut last_shot_time)) = ship.get_single_mut() {
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
            let time_elapsed = time.elapsed();
            if time_elapsed - last_shot_time.0 > SHOT_SPACING {
                spawn_bullet(
                    commands,
                    bullet_assets,
                    position.0,
                    rotation.0,
                    time
                );
                last_shot_time.0 = time_elapsed;
            }
        }
    }
}

pub struct AsteroidsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct ObjectUpdate;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::new().with_rng_seed(WORLD_SEED));
        app.add_systems(Startup, (
            spawn_camera,
            spawn_ship,
            load_spawner,
            load_bullet,
            load_asteroids.after(load_spawner),
            spawn_asteroid_random.after(load_asteroids),
        ));
        app.add_systems(Update, (
            handle_player_input,
            //spawn_asteroid_random.run_if(on_timer(Duration::from_secs(2))),
            move_ship,
            move_bullets,
            move_asteroids,
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