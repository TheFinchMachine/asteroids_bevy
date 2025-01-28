use bevy::{prelude::*, scene};
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    shape: Shape,
    position: Position,
    velocity: Velocity,
}

impl ShipBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ship: Ship,
            shape: Shape(Vec2::new(10., 10.)),
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::new(0., 0.)),
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
        ShipBundle::new(50., 50.),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_scale(Vec3::new(50., 50., 50.)),
    ));
}

fn create_ship(scale: f32) -> Mesh {
    // Create a new mesh using a triangle list topology, where each set of 3 vertices composes a triangle.
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        // Add 4 vertices, each with its own position attribute (coordinate in
        // 3D space), for each of the corners of the parallelogram.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 1.*scale, 0.0], [1.*scale, -1.5*scale, 0.0], [0.0, -1.*scale, 0.0], [-1.*scale, -1.5*scale, 0.0]]
        )
        // Assign a UV coordinate to each vertex.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 1.0], [0.5, 0.0], [1.0, 0.0], [0.5, 1.0]]
        )
        // Assign normals (everything points outwards)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]
        )
        // After defining all the vertices and their attributes, build each triangle using the
        // indices of the vertices that make it up in a counter-clockwise order.
        .with_inserted_indices(mesh::Indices::U32(vec![
            // First triangle
            0, 2, 1,
            // Second triangle
            3, 0, 2
        ]))
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(Camera2d);
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            spawn_camera,
            spawn_ship,

        ));
        app.add_systems(Update, (
            project_positions,
        ));
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(AsteroidsPlugin)
    .run();
}