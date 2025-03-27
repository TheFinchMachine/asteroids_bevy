use crate::schedule::InGameSet;
use bevy::prelude::*;

#[derive(Component, Debug)]
struct RestartMessage;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    InGame,
    Paused,
    GameOver,
}

fn pause_system(
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::InGame => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::InGame);
            }
            _ => (),
        }
    }
}

fn restart_game(
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        match state.get() {
            GameState::GameOver => {
                next_state.set(GameState::InGame);
            }
            _ => (),
        }
    }
}

fn spawn_restart_message(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let text_height = window_height / 4.0 - 18.0;

        let font = asset_server.load("fonts/FiraMono-Medium.ttf");
        let text_font = TextFont {
            font,
            font_size: 36.0,
            ..default()
        };

        commands.spawn((
            RestartMessage,
            Text2d::new("Press R to Restart"),
            text_font.clone(),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(0.0, text_height, 0.0)),
        ));
    }
}

fn despawn_restart_message(mut commands: Commands, messages: Query<Entity, With<RestartMessage>>) {
    for entity in messages.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(
            Update,
            (pause_system, restart_game).in_set(InGameSet::MenuInput),
        );
        app.add_systems(OnEnter(GameState::GameOver), spawn_restart_message);
        app.add_systems(OnExit(GameState::GameOver), despawn_restart_message);
    }
}
