use crate::{schedule::InGameSet, GameState};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

// TODO! add teams to score
#[derive(Resource, Default)]
struct Score {
    score: u16,
}

#[derive(Asset, Default, Deserialize, Clone, Copy, TypePath)]
struct ScoreConfig {
    font_size: f32,
    margin: f32,
}

#[derive(Resource)]
struct ScoreConfigHandle {
    config: Handle<ScoreConfig>,
}

fn load_config(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let config = asset_server.load("a.score.ron");
    commands.insert_resource(ScoreConfigHandle {
        config,
    });
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

#[derive(Component)]
struct NeedsScoreboard;

fn spawn_playerscore (
    mut commands: Commands,
) {
    commands.spawn((PlayerScore, NeedsScoreboard));
}

fn spawn_scoreboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
    configs: Res<Assets<ScoreConfig>>,
    config_handle: Res<ScoreConfigHandle>,
    mut scoreboards: Query<(Entity, &mut NeedsScoreboard)>,
) {
    // only spawn once...
    if let Ok(window) = window.get_single() {
        if let Some(config) = configs.get(config_handle.config.id()) {
            for (entity, _) in scoreboards.iter_mut() {
                let window_height = window.resolution.height();
                let text_height = window_height / 2.0 - config.margin;

                let font = asset_server.load("fonts/FiraMono-Medium.ttf");
                let text_font = TextFont {
                    font,
                    font_size: config.font_size,
                    ..default()
                };

                //needs to append, not spawn
                commands.entity(entity).insert((
                    Text2d::new("0"),
                    text_font.clone(),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_translation(Vec3::new(0.0, text_height, 0.0)),
                ));

                //remove scoreboard
                commands.entity(entity).remove::<NeedsScoreboard>();
            }
        }
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
        app.add_plugins(RonAssetPlugin::<ScoreConfig>::new(&["score.ron"]));
        app.init_resource::<Score>();
        app.add_event::<Scored>();
        app.add_systems(Startup, (load_config, spawn_playerscore));
        app.add_systems(Update, (spawn_scoreboard).in_set(InGameSet::LoadEntities));
        app.add_systems(
            Update,
            (update_score, update_scoreboard).in_set(InGameSet::UpdateEntities),
        );
        app.add_systems(OnEnter(GameState::GameOver), reset_score);
    }
}
