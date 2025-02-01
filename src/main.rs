use bevy::input::keyboard::Key;
use bevy::math::NormedVectorSpace;
use bevy::{prelude::*, scene, window};
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::time::common_conditions::on_timer;
use bevy::sprite::{Wireframe2dPlugin, Wireframe2dConfig, Wireframe2d};
use std::time::Duration;
use bevy_turborand::prelude::*;


const WORLD_SEED: u64 = 1024;

#[derive(Component)]
struct TimeStamp(Duration);

// don't use Rot2 as it is effectively a 2d quat. 2d rots don't suffer from gimbal lock, so we don't need that complexity.
#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct AngularVelocity(f32);

#[derive(Component)]
struct AngularAcceleration(f32);

#[derive(Component)]
struct AngularDamping(f32);

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Damping(f32);

#[derive(Component)]
struct Scale(f32);

#[derive(Component)]
struct RigidBody{
    radius: f32,
    mass: f32,
}

fn collision_bounce(
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

    let vel1_normal_new = (vel1_normal * (mass1 - mass2) + 2.0 * mass2* vel2_normal) / (mass1 + mass2);
    let vel2_normal_new = (vel2_normal * (mass2 - mass1) + 2.0 * mass1 * vel1_normal) / (mass1 + mass2);

    ((tangent * vel1_tangent) + (normal * vel1_normal_new), (tangent * vel2_tangent) + (normal * vel2_normal_new))

}

fn collide (
    pos1: Vec2,
    pos2: Vec2,
    r1: f32,
    r2: f32,
) -> (Vec2, f32, f32) {
    let dir = pos2 - pos1;
    let dist = dir.length().abs();
    let collide_dist = r1 + r2;
    (dir, dist, collide_dist)
}

fn collisions_asteroids (
    mut bodies: Query<(&mut Position, &mut Velocity, &RigidBody), With<Asteroid>>,
) {
    let mut combinations = bodies.iter_combinations_mut();
    while let Some([(mut pos1, mut vel1, body1), (mut pos2, mut vel2, body2)]) = combinations.fetch_next() {
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

fn collisions_ship(
    mut commands: Commands,
    mut ships: Query<(Entity, &Position, &RigidBody), With<Ship>>,
    asteroids: Query<(&Position, &RigidBody), With<Asteroid>>,
) {
    for(ship_entity, ship_pos, ship_body) in &ships {
        for(ast_pos, ast_body) in &asteroids {
            let (dir, dist, collide_dist) = collide(ship_pos.0, ast_pos.0, ship_body.radius, ast_body.radius);
            if dist < collide_dist {
                commands.entity(ship_entity).despawn();
            }
        }
    }
}
const SHIP_SPEED: f32 = 2.0;
const SHIP_DAMPING: f32 = 1.0;

const SHIP_SPEED_ANGULAR: f32 = 6.0;
const SHIP_DAMPING_ANGULAR: f32 = 10.0;

#[derive(Component)]
struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
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
    rigid_body: RigidBody
}

impl ShipBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ship: Ship,
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
            rigid_body: RigidBody{radius: 0.1, mass: 2.0}
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
        Transform::default(),
    ));
}

fn move_ship(
    time: Res<Time>,
    mut ship: Query<(
        &mut Position, &mut Velocity, &Acceleration,
        &mut Rotation, &mut AngularVelocity, &AngularAcceleration), With<Ship>>
) {
    for (mut position, mut velocity, acceleration,
        mut rotation, mut angular_velocity, angular_acceleration,) in &mut ship {
        //scale acceleration and velocity and damping
        angular_velocity.0 += angular_acceleration.0 * SHIP_SPEED_ANGULAR * time.delta_secs();
        rotation.0 += angular_velocity.0 * time.delta_secs();
        angular_velocity.0 *= (-SHIP_DAMPING_ANGULAR*time.delta_secs()).exp();

        let rotator = Rot2::radians(rotation.0);

        //scale acceleration and velocity and damping
        velocity.0 += rotator * acceleration.0  * SHIP_SPEED * time.delta_secs();
        position.0 += velocity.0 * time.delta_secs();
        velocity.0 *= (-SHIP_DAMPING*time.delta_secs()).exp();
    }
}

fn move_obj(
    time: Res<Time>,
    mut obj: Query<(&mut Position, &mut Rotation, &Velocity, &AngularVelocity), Without<Ship>>
) {
    for (mut position, mut rotation, velocity, angular_velocity) in &mut obj {
        position.0 += velocity.0*time.delta_secs();
        rotation.0 += angular_velocity.0*time.delta_secs();
    }
}

const BULLET_SPEED: f32 = 6.0;
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
    position: Position,
    rotation: Rotation,
    velocity: Velocity,
    angular_velocity: AngularVelocity,
    scale: Scale,
    spawn_time: TimeStamp,
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

#[derive(Bundle)]
struct AsteroidBundle {
    asteroid: Asteroid,
    position: Position,
    velocity: Velocity,
    rotation: Rotation,
    angular_velocity: AngularVelocity,
    scale: Scale,
    rigid_body: RigidBody,
}

impl AsteroidBundle {
    fn new(position: Vec2, velocity: Vec2, angular_velocity: f32, scale: f32,) -> Self {
        Self {
            asteroid: Asteroid,
            position: Position(position),
            velocity: Velocity(velocity),
            scale: Scale(scale),
            rotation: Rotation(0.0),
            angular_velocity: AngularVelocity(angular_velocity),
            rigid_body: RigidBody{radius: scale*0.012, mass: 2.0},
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
    angular_velocity: f32,
    scale: f32,
) {
    let mesh = spawner.rng.usize(0..ASTEROID_VARIANTS);
    commands.spawn((
        AsteroidBundle::new(position, velocity, angular_velocity, scale),
        Mesh2d(asteroid_assets.meshes[mesh].clone()),
        MeshMaterial2d(asteroid_assets.material.clone()),
        Transform::default(),
    ));
}


fn spawn_asteroid_random(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
) {
    for _ in 0..5{
        let position = Vec2::new(spawner.rng.f32_normalized()*GRID_SIZE*0.5, spawner.rng.f32_normalized()*GRID_SIZE*0.5);
        let velocity = Vec2::new(spawner.rng.f32_normalized()*3.0, spawner.rng.f32_normalized()*3.0);
        let scale = spawner.rng.f32()*5.0 + 45.0;
        let angular_velocity = spawner.rng.f32_normalized()*1.0;

        spawn_asteroid(&mut commands, &asteroid_assets, &mut spawner, position, velocity, angular_velocity, scale);
    }
}

fn create_astroid_mesh(spawner: &mut ResMut<SpawnGenerator>) -> Mesh {
    let rng = &mut spawner.rng;
    // create semi-random circle
    let num_verts = rng.usize(8..12);
    let angle_step = 360.0/num_verts as f32;
    let angle_range = angle_step * 0.1;
    let mut positions = Vec::with_capacity(num_verts);
    
    for i in 0..num_verts {
        let radius = rng.f32_normalized()*0.1 + 1.0;
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
const GRID_SIZE: f32 = 10.;
fn project_positions(
    mut positionables: Query<(&mut Transform, &Position, &Rotation, &Scale)>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let window_width = window.resolution.width();

        let grid_to_height = window_height / GRID_SIZE;
        let grid_to_width = window_width / GRID_SIZE;

        for (mut transform, position, rotation, scale) in &mut positionables {
            let mut new_position = position.0;
            new_position.x *= grid_to_width;
            new_position.y *= grid_to_height;
            //wrap objects around the screen
            //new_position.x = wrap_around(new_position.x, -window_width/2.0, window_width);
            //new_position.y = wrap_around(new_position.y, -window_height/2.0, window_height);
            //println!("new_position.y: {}", new_position.y);
            transform.translation = new_position.extend(0.);

            transform.rotation = Quat::from_rotation_z(rotation.0);

            transform.scale = Vec3::new(scale.0, scale.0, scale.0)
        }
    }
}

fn wrap_obj(
    mut obj: Query<&mut Position>,

) {
    let grid_extends = 0.5;
    for (mut position) in &mut obj {
        position.0.x = wrap_around(position.0.x, -GRID_SIZE*0.5 - grid_extends, GRID_SIZE + (2.0*grid_extends));
        position.0.y = wrap_around(position.0.y, -GRID_SIZE*0.5 - grid_extends, GRID_SIZE + (2.0*grid_extends));
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
            acceleration.0.y = SHIP_SPEED;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            acceleration.0.y = -SHIP_SPEED;
        } else {
            acceleration.0.y = 0.;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            angular_acceleration.0 = -SHIP_SPEED_ANGULAR;
        } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
            angular_acceleration.0 = SHIP_SPEED_ANGULAR;
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
            move_obj,
            move_ship,
            wrap_obj,
            collisions_asteroids,
            collisions_ship,
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