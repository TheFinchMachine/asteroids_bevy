use bevy::prelude::*;

#[derive(Resource, Default)]
struct Score {
    score: u16,
}

#[derive(Event)]
pub struct Scored;

fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for _ in events.read() {
        score.score += 1;
    }
}

const SCOREBOARD_FONT_SIZE: f32 = 36.0;
const SCOREBOARD_TEXT_MARGIN: f32 = 18.0;

#[derive(Component)]
struct PlayerScore;

fn spawn_scoreboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        let text_height = window_height / 2. - SCOREBOARD_TEXT_MARGIN;

        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_font = TextFont {
            font,
            font_size: SCOREBOARD_FONT_SIZE,
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
        app.init_resource::<Score>();
        app.add_event::<Scored>();
        app.add_systems(Startup, spawn_scoreboard);
        app.add_systems(Update, (update_score, update_scoreboard));
    }
}
