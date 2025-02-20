use crate::bodies::*;
use crate::grid::*;
use crate::load_spawner;
use crate::schedule::InGameSet;
use crate::score::Scored;
use crate::spawner::SpawnGenerator;
use crate::GameState;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_turborand::prelude::*;
use serde::Deserialize;
use std::time::Duration;
//use web_sys::console;


#[derive(Deserialize, Asset, Clone, Copy, TypePath)]
struct AsteroidConfig {
    varients: usize,
    num_verts: (usize, usize),
    angle_range: f32,
    radius_range: f32,
    radius_base: f32,
}

#[derive(Resource)]
struct AsteroidConfigHandle {
    config: Handle<AsteroidConfig>,
}

#[derive(Resource)]
struct AsteroidAssets {
    meshes: Vec<Handle<Mesh>>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Bundle)]
struct AsteroidBundle {
    asteroid: Asteroid,
    position: Position,
    velocity: Velocity,
    rotation: Rotation,
    angular_velocity: AngularVelocity,
    scale: Scale,
    rigid_body: RigidBody,
    collider: Collider,
}

impl AsteroidBundle {
    fn new(position: Vec2, velocity: Vec2, angular_velocity: f32, scale: f32) -> Self {
        Self {
            asteroid: Asteroid,
            position: Position(position),
            velocity: Velocity(velocity),
            scale: Scale(scale),
            rotation: Rotation(0.0),
            angular_velocity: AngularVelocity(angular_velocity),
            rigid_body: RigidBody {
                radius: scale * 0.01,
                mass: 2.0,
            },
            collider: Collider { team: 0 },
        }
    }
}

fn load_config(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let config = asset_server.load("a.ast.ron");
    commands.insert_resource(AsteroidConfigHandle {
        config,
    });
}

fn load_asteroids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawner: ResMut<SpawnGenerator>,
    configs: Res<Assets<AsteroidConfig>>,
    config_handle: Res<AsteroidConfigHandle>,
    asteroid_assets: Option<Res<AsteroidAssets>>,
) {
    if asteroid_assets.is_some(){
        return;
    }
    if let Some(config) = configs.get(config_handle.config.id()) {
        let material = materials.add(Color::srgb(0.5, 1., 0.5));

        let mut new_meshes = Vec::with_capacity(config.varients);
        for _ in 0..config.varients {
            new_meshes.push(meshes.add(create_astroid_mesh(&mut spawner, &config)));
        }

        commands.insert_resource(AsteroidAssets {
            meshes: new_meshes,
            material,
        });
    }
}

fn spawn_asteroid(
    commands: &mut Commands,
    asteroid_assets: &Res<AsteroidAssets>,
    spawner: &mut ResMut<SpawnGenerator>,
    config: &AsteroidConfig,
    position: Vec2,
    velocity: Vec2,
    angular_velocity: f32,
    scale: f32,
) {
    let mesh = spawner.rng.usize(0..config.varients);
    commands.spawn((
        AsteroidBundle::new(position, velocity, angular_velocity, scale),
        Mesh2d(asteroid_assets.meshes[mesh].clone()),
        MeshMaterial2d(asteroid_assets.material.clone()),
        Transform::default(),
    ));
}

fn spawn_asteroid_child(
    commands: &mut Commands,
    asteroid_assets: &Res<AsteroidAssets>,
    spawner: &mut ResMut<SpawnGenerator>,
    config: &AsteroidConfig,
    position: Vec2,
    velocity: Vec2,
    scale: f32,
    offset: f32,
) {
    let vel_len = velocity.length();
    let vel_offset1 = Rot2::degrees(180.0-offset) * velocity.normalize();
    let ang_vel = spawner.rng.f32_normalized();
    spawn_asteroid(
        commands,
        asteroid_assets,
        spawner,
        config,
        position + vel_offset1 * scale * 0.0005,
        vel_offset1 * vel_len * -0.75,
        ang_vel,
        scale / 1.5,
    );
}

fn create_astroid_mesh(spawner: &mut ResMut<SpawnGenerator>, config: &AsteroidConfig) -> Mesh {
    let rng = &mut spawner.rng;
    // create semi-random circle
    let num_verts = rng.usize(config.num_verts.0..config.num_verts.1);
    let angle_step = 360.0 / num_verts as f32;
    let angle_range = angle_step * config.angle_range;
    let mut positions = Vec::with_capacity(num_verts);

    for i in 0..num_verts {
        let radius = rng.f32_normalized() * config.radius_range + config.radius_base;
        let angle = rng.f32_normalized() * (i as f32 * angle_range) + (i as f32 * angle_step);
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
        let mut normal;
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
        let max = num_verts * 2;
        //triangle 1 cw, which is wrong
        indices.push((i % max) as u32);
        indices.push(((i + num_verts) % max) as u32);
        indices.push(((i + 1) % num_verts) as u32);

        //triangle 2 cw, which is wrong
        indices.push(((i + num_verts) % max) as u32);
        indices.push(((i + 1) % num_verts + num_verts) as u32);
        indices.push(((i + 1) % num_verts) as u32);
    }

    let normals_3d = vec![[0.0, 0.0, 1.0]; num_verts * 2];

    // build mesh
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions_3d)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals_3d)
    .with_inserted_indices(mesh::Indices::U32(indices))
}

fn spawn_asteroid_random(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
    grid: Res<Grid>,
    configs: Res<Assets<AsteroidConfig>>,
    config_handle: Res<AsteroidConfigHandle>,
) {
    // spawn position offscreen inside grid extents
    let x_dist = spawner.rng.f32_normalized() * grid.extends;
    let y_dist = spawner.rng.f32_normalized() * grid.extends;
    let x = if x_dist < 0.0 {
        x_dist - grid.width_half
    } else {
        x_dist + grid.width_half
    };
    let y = if y_dist < 0.0 {
        y_dist - grid.height_half
    } else {
        y_dist + grid.height_half
    };
    let position = Vec2::new(x, y);

    let velocity = Vec2::new(
        spawner.rng.f32_normalized() * 2.0,
        spawner.rng.f32_normalized() * 2.0,
    );
    let scale = spawner.rng.f32() * 5.0 + 45.0;
    let angular_velocity = spawner.rng.f32_normalized() * 1.0;

    if let Some(config) = configs.get(config_handle.config.id()) {
        spawn_asteroid(
            &mut commands,
            &asteroid_assets,
            &mut spawner,
            &config,
            position,
            velocity,
            angular_velocity,
            scale,
        );
    }
}

// TODO! switch spawning children to an event
fn destroy_asteroids(
    mut commands: Commands,
    asteroid_assets: Option<Res<AsteroidAssets>>,
    mut spawner: ResMut<SpawnGenerator>,
    asteroids: Query<(Entity, &Collider, &Position, &Velocity, &Scale), With<Asteroid>>,
    colliders: Query<&Collider>,
    mut collisions: EventReader<Collision>,
    mut score: EventWriter<Scored>,
    configs: Res<Assets<AsteroidConfig>>,
    config_handle: Res<AsteroidConfigHandle>,
) {
    if let Some(config) = configs.get(config_handle.config.id()) {
        if let Some(assets) = asteroid_assets {
            for event in collisions.read() {
                for (entity_a, entity_b) in [
                    (event.entity1, event.entity2),
                    (event.entity2, event.entity1),
                ] {
                    if let Ok((ast_entity, ast_collider, ast_pos, ast_vel, ast_scale)) =
                        asteroids.get(entity_a)
                    {
                        if let Ok(collider) = colliders.get(entity_b) {
                            if collider.team != ast_collider.team {
                                // TODO! add teams to score
                                score.send(Scored);
                                if ast_scale.0 > 25.0 {
                                    spawn_asteroid_child(
                                        &mut commands,
                                        &assets,
                                        &mut spawner,
                                        &config,
                                        ast_pos.0,
                                        ast_vel.0,
                                        ast_scale.0,
                                        50.0,
                                    );
                                    spawn_asteroid_child(
                                        &mut commands,
                                        &assets,
                                        &mut spawner,
                                        &config,
                                        ast_pos.0,
                                        ast_vel.0,
                                        ast_scale.0,
                                        -50.0,
                                    );
                                }
                                commands.entity(ast_entity).despawn();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn bounce_asteroids(
    mut asteroids: Query<(&mut Position, &mut Velocity, &RigidBody), With<Asteroid>>,
    mut collisions: EventReader<Collision>,
) {
    for event in collisions.read() {
        if let Ok(
            [(mut ast_a_pos, mut ast_a_vel, ast_a_body), (mut ast_b_pos, mut ast_b_vel, ast_b_body)],
        ) = asteroids.get_many_mut([event.entity1, event.entity2])
        {
            //console::log_1(&"Received CollisionEvent".into());

            /*console::log_1(&format!(
                "Before Collision: A Vel: {:?}, B Vel: {:?}",
                ast_a_vel.0, ast_b_vel.0
            )
            .into());*/
            let normal = event.dir.normalize();
            (ast_a_vel.0, ast_b_vel.0) = collision_bounce(
                ast_a_vel.0,
                ast_b_vel.0,
                normal,
                ast_a_body.mass,
                ast_b_body.mass,
            );
            /*console::log_1(&format!(
                "After Collision: A Vel: {:?}, B Vel: {:?}",
                ast_a_vel.0, ast_b_vel.0
            )
            .into());*/

            let depth = event.collide_dist - event.dist;
            let correction = normal * (depth * 0.8);
            ast_a_pos.0 -= correction;
            ast_b_pos.0 += correction;
        }
    }
}

fn despawn_asteroids(mut commands: Commands, asteroids: Query<Entity, With<Asteroid>>) {
    for entity in asteroids.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<AsteroidConfig>::new(&["ast.ron"]));
        app.add_systems(Startup, (load_spawner, load_config));
        app.add_systems(Update, (load_asteroids).in_set(InGameSet::LoadEntities));
        app.add_systems(
            Update,
            (destroy_asteroids).in_set(InGameSet::DespawnEntities),
        );
        app.add_systems(
            Update,
            (
                spawn_asteroid_random.run_if(on_timer(Duration::from_secs(2))),
            )
                .in_set(InGameSet::UpdateEntities),
        );
        app.add_systems(Update, (bounce_asteroids).in_set(InGameSet::CollisionReaction));
        app.add_systems(OnEnter(GameState::GameOver), despawn_asteroids);
    }
}
