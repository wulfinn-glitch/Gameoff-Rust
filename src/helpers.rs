use bevy::diagnostic::{Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use crate::enemy::Enemy;
use crate::player::Player;

#[derive(Component)]
pub struct FpsText;

#[derive(Resource)]
pub struct FpsUpdateTimer(Timer);

pub fn setup_fps_counter(mut commands: Commands) {
    commands.insert_resource(
        FpsUpdateTimer(Timer::from_seconds(0.25, TimerMode::Repeating))
    );
    commands.spawn((
        FpsText,
        Node{
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("FPS 0.0".to_string()),
        TextFont{
            font_size: 10.0,
            ..default()
        }
    ));
}

pub fn show_fps(
    time: Res<Time>,
    mut timer: ResMut<FpsUpdateTimer>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("FPS {:.1}", value);
            }
        }
    }
}
