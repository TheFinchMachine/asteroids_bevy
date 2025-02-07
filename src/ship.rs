use crate::bodies::*;
use bevy::prelude::*;
use std::time::Duration;

pub const SHIP_SPEED: f32 = 2.0;
const SHIP_DAMPING: f32 = 1.0;

pub const SHIP_SPEED_ANGULAR: f32 = 6.0;
const SHIP_DAMPING_ANGULAR: f32 = 10.0;

#[derive(Component)]
pub struct Ship;

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    position: Position,
    rotation: Rotation,
    scale: Scale,
    velocity: Velocity,
    acceleration: Acceleration,
    angular_velocity: AngularVelocity,
    angular_acceleration: AngularAcceleration,
    last_shot: TimeStamp,
    rigid_body: RigidBody,
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
            angular_velocity: AngularVelocity(0.0),
            angular_acceleration: AngularAcceleration(0.0),
            last_shot: TimeStamp(Duration::ZERO),
            rigid_body: RigidBody {
                radius: 0.1,
                mass: 2.0,
            },
        }
    }
}

pub fn spawn_ship(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: 0,
            primitive: 0,
        }
        .from_asset("meshes/ship.glb"),
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
/*
pub fn move_ship(
    time: Res<Time>,
    mut ship: Query<
        (
            &mut Position,
            &mut Velocity,
            &Acceleration,
            &mut Rotation,
            &mut AngularVelocity,
            &AngularAcceleration,
        ),
        With<Ship>,
    >,
) {
    for (
        mut position,
        mut velocity,
        acceleration,
        mut rotation,
        mut angular_velocity,
        angular_acceleration,
    ) in &mut ship
    {
        //scale acceleration and velocity and damping
        angular_velocity.0 += angular_acceleration.0 * SHIP_SPEED_ANGULAR * time.delta_secs();
        rotation.0 += angular_velocity.0 * time.delta_secs();
        angular_velocity.0 *= (-SHIP_DAMPING_ANGULAR * time.delta_secs()).exp();

        let rotator = Rot2::radians(rotation.0);

        //scale acceleration and velocity and damping
        velocity.0 += rotator * acceleration.0 * SHIP_SPEED * time.delta_secs();
        position.0 += velocity.0 * time.delta_secs();
        velocity.0 *= (-SHIP_DAMPING * time.delta_secs()).exp();
    }
}
*/
