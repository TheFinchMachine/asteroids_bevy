use asteroids::AsteroidsGamePlugin;
use bevy::prelude::*;

// test workflow
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AsteroidsGamePlugin)
        .run();
}
