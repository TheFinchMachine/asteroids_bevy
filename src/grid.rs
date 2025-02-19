use crate::{bodies::*, schedule::InGameSet};
use bevy::{prelude::*, window::WindowResized};
use serde::Deserialize;

// because coords staring in center, half height and with make much more sense
#[derive(Resource)]
pub struct Grid {
    pub size: f32,
    pub extends: f32,
    pub height_half: f32,
    pub width_half: f32,
}

// so velocity numbers make sense
fn grid_build(mut commands: Commands, window: Query<&Window>) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let window_width = window.resolution.width();
        let window_scale = window.resolution.scale_factor();

        //TODO! convert to config
        let size = 100.0 * window_scale;
        commands.insert_resource(Grid {
            size,
            extends: 0.5 * window_scale,
            height_half: window_height * 0.5 / size,
            width_half: window_width * 0.5 / size,
        });
    }
}

fn grid_update(width: f32, height: f32, grid: &mut ResMut<Grid>) {
    grid.width_half = width * 0.5 / grid.size;
    grid.height_half = height * 0.5 / grid.size;
}

fn on_resize(mut resize_reader: EventReader<WindowResized>, mut grid: ResMut<Grid>) {
    for e in resize_reader.read() {
        grid_update(e.width, e.height, &mut grid);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2d);
}

fn project_positions(
    mut positionables: Query<(&mut Transform, &Position, &Rotation, &Scale)>,
    grid: Res<Grid>,
) {
    for (mut transform, position, rotation, scale) in &mut positionables {
        let mut new_position = position.0;
        new_position.x *= grid.size;
        new_position.y *= grid.size;

        //wrap objects around the screen
        transform.translation = new_position.extend(0.);

        transform.rotation = Quat::from_rotation_z(rotation.0);

        transform.scale = Vec3::new(scale.0, scale.0, scale.0)
    }
}

fn wrap_obj(mut obj: Query<&mut Position>, grid: Res<Grid>) {
    for mut position in &mut obj {
        position.0.x = wrap_around(
            position.0.x,
            -grid.width_half - grid.extends,
            grid.width_half * 2.0 + (2.0 * grid.extends),
        );
        position.0.y = wrap_around(
            position.0.y,
            -grid.height_half - grid.extends,
            grid.height_half * 2.0 + (2.0 * grid.extends),
        );
    }
}

fn wrap_around(value: f32, min_value: f32, range: f32) -> f32 {
    // modulo preserves sign so we need to add range and then modulo again to handle negatives
    // could also be done with an if statement but this is specifically branchless
    ((value - min_value) % range + range) % range + min_value
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, grid_build));
        app.add_systems(Update, (wrap_obj).in_set(InGameSet::UpdateEntities));
        app.add_systems(Update, (on_resize).in_set(InGameSet::MenuInput));
        app.add_systems(Update, (project_positions).in_set(InGameSet::RenderSetup));
    }
}
