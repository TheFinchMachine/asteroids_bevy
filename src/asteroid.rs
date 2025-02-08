use crate::bodies::*;
use crate::grid::*;
use crate::spawner::SpawnGenerator;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_turborand::prelude::*;

const ASTEROID_VARIANTS: usize = 100;

#[derive(Resource)]
pub struct AsteroidAssets {
    pub meshes: [Handle<Mesh>; ASTEROID_VARIANTS],
    pub material: Handle<ColorMaterial>,
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

pub fn load_asteroids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawner: ResMut<SpawnGenerator>,
) {
    let material = materials.add(Color::srgb(0.5, 1., 0.5));

    let new_meshes: [Handle<Mesh>; ASTEROID_VARIANTS] =
        std::array::from_fn(|_| meshes.add(create_astroid_mesh(&mut spawner)));

    commands.insert_resource(AsteroidAssets {
        meshes: new_meshes,
        material,
    });
}

pub fn spawn_asteroid(
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

pub fn spawn_asteroid_child(
    commands: &mut Commands,
    asteroid_assets: &Res<AsteroidAssets>,
    spawner: &mut ResMut<SpawnGenerator>,
    position: Vec2,
    velocity: Vec2,
    scale: f32,
    offset: f32,
) {
    let vel_len = velocity.length();
    let vel_offset1 = Rot2::degrees(offset) * velocity.normalize();
    let ang_vel = spawner.rng.f32_normalized();
    spawn_asteroid(
        commands,
        asteroid_assets,
        spawner,
        position + vel_offset1 * scale * 0.001,
        vel_offset1 * vel_len * 0.75,
        ang_vel,
        scale / 1.5,
    );
}

fn create_astroid_mesh(spawner: &mut ResMut<SpawnGenerator>) -> Mesh {
    let rng = &mut spawner.rng;
    // create semi-random circle
    let num_verts = rng.usize(8..12);
    let angle_step = 360.0 / num_verts as f32;
    let angle_range = angle_step * 0.0;
    let mut positions = Vec::with_capacity(num_verts);

    for i in 0..num_verts {
        let radius = rng.f32_normalized() * 0.25 + 0.75;
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

pub fn spawn_asteroid_random(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
    grid: Res<Grid>,
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

    spawn_asteroid(
        &mut commands,
        &asteroid_assets,
        &mut spawner,
        position,
        velocity,
        angular_velocity,
        scale,
    );
}

fn destroy_asteroids(
    mut commands: Commands,
    asteroid_assets: Res<AsteroidAssets>,
    mut spawner: ResMut<SpawnGenerator>,
    asteroids: Query<(Entity, &Collider, &Position, &Velocity, &Scale), With<Asteroid>>,
    colliders: Query<&Collider>,
    mut collisions: EventReader<Collision>,
) {
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
                        if ast_scale.0 > 25.0 {
                            spawn_asteroid_child(
                                &mut commands,
                                &asteroid_assets,
                                &mut spawner,
                                ast_pos.0,
                                ast_vel.0,
                                ast_scale.0,
                                50.0,
                            );
                            spawn_asteroid_child(
                                &mut commands,
                                &asteroid_assets,
                                &mut spawner,
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

fn bounce_asteroids(
    mut asteroids: Query<(&mut Position, &mut Velocity, &RigidBody), With<Asteroid>>,
    mut collisions: EventReader<Collision>,
) {
    for event in collisions.read() {
        if let Ok(
            [(mut ast_a_pos, mut ast_a_vel, ast_a_body), (mut ast_b_pos, mut ast_b_vel, ast_b_body)],
        ) = asteroids.get_many_mut([event.entity1, event.entity2])
        {
            let normal = event.dir.normalize();
            (ast_a_vel.0, ast_b_vel.0) = collision_bounce(
                ast_a_vel.0,
                ast_b_vel.0,
                normal,
                ast_a_body.mass,
                ast_b_body.mass,
            );

            let depth = event.collide_dist - event.dist;
            let correction = normal * (depth * 0.5);
            ast_a_pos.0 -= correction;
            ast_b_pos.0 += correction;
        }
    }
}
