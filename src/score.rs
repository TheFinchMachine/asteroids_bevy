use crate::{schedule::InGameSet, GameState};
use bevy::prelude::*;
use bevy_easy_config::EasyConfigPlugin;
use serde::Deserialize;

// TODO! add teams to score
#[derive(Resource, Default)]
struct Score {
    score: u16,
}

#[derive(Resource, Default, Deserialize, Asset, Clone, Copy, TypePath)]
struct ScoreConfig {
    font_size: f32,
    margin: f32,
}

#[derive(Event)]
pub struct Scored;

fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for _ in events.read() {
        score.score += 1;
    }
}

fn reset_score(mut score: ResMut<Score>) {
    score.score = 0;
}

#[derive(Component)]
struct PlayerScore;

fn spawn_scoreboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
    config: Res<ScoreConfig>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let text_height = window_height / 2.0 - config.margin;

        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_font = TextFont {
            font,
            font_size: config.font_size,
            ..default()
        };

        commands.spawn((
            PlayerScore,
            Text2d::new("0"),
            text_font.clone(),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_translation(Vec3::new(0.0, text_height, 0.0)),
        ));
    }
}

fn update_scoreboard(mut player_score: Query<&mut Text2d, With<PlayerScore>>, score: Res<Score>) {
    if score.is_changed() {
        if let Ok(mut player_score) = player_score.get_single_mut() {
            player_score.0 = score.score.to_string();
        }
    }
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EasyConfigPlugin::<ScoreConfig>::new("score.cfg.ron"));
        app.init_resource::<Score>();
        app.add_event::<Scored>();
        app.add_systems(Startup, spawn_scoreboard);
        app.add_systems(
            Update,
            (update_score, update_scoreboard).in_set(InGameSet::EntityUpdates),
        );
        app.add_systems(OnEnter(GameState::GameOver), reset_score);
    }
}
