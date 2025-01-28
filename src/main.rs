use bevy::input::keyboard::Key;
use bevy::{prelude::*, scene};
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Shape(Vec2);

const SHIP_SPEED: f32 = 0.15;

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
        ShipBundle::new(0., 0.),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_scale(Vec3::new(10., 10., 10.)),
    ));
}

fn move_ship(
    mut ship: Query<(&mut Position, &Velocity), With<Ship>>
) {
    for (mut position, velocity) in &mut ship {
        position.0 = position.0 + velocity.0 * SHIP_SPEED;

    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_empty()
        .insert(Camera2d);
}

const GRID_SIZE: f32 = 100.;
fn project_positions(
    mut positionables: Query<(&mut Transform, &Position)>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let window_width = window.resolution.width();

        //let window_aspect = window_width / window_height;

        for (mut transform, position) in &mut positionables {
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
        }
    }
}

fn wrap_around(value: f32, min_value: f32, range: f32) -> f32 {
    // modulo preserves sign so we need to add range and then modulo again to handle negatives
    // could also be done with an if statement but this is specifically branchless
    // assuming modulo implementation is branchless.
    // may be possible to improve by precomputing 1/range and then using a fast modulo
    // because range only changes on window size change
    ((value - min_value) % range + range) % range + min_value
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ship: Query<&mut Velocity, With<Ship>>,
) {
    if let Ok(mut velocity) = ship.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.0.y = -1.;
        } else {
            velocity.0.y = 0.;
        }
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
            handle_player_input,
            move_ship,
            project_positions.after(move_ship),
        ));
    }
}

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(AsteroidsPlugin)
    .run();
}